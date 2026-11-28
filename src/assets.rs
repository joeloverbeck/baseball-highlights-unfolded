//! Admission for the owner-local original collection and its approved proxy faces.
use crate::{CardId, GloveReview, series::buy_round_face_ids};
use image::DynamicImage;
use serde::Deserialize;
use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
};
use unfolded_assets::{read_bounded, regular_file, verify_bytes};

const ORIGINAL_MANIFEST_HASH: &str =
    "c1c528e5ca696ba82c35465b4fb98d57bf830f27376c2cd77582047c54d04ede";
const PHOTOCRAFT_MANIFEST_HASH: &str =
    "8d47e5902757a8d475b831e6f0ea03744c9b29d4d7e06193a8525fe596644481";
const RECOVERED_MANIFEST_HASH: &str =
    "d858a56885c312cd45c22c45868d81692acacfe5f92d2ba7a591a801d68d40a3";
const LEDGER: &str = include_str!("../docs/foundation/card-content.csv");
const BACK: &str = "images/card-backs/backurl-omruwvLU2jpg.jpg";
const RECOVERED_STARTERS: &str = "catalog/recovered-starters.json";

#[derive(Deserialize)]
struct CatalogEntry {
    card_id: u32,
    face: String,
    back: String,
}

pub struct Collection {
    root: PathBuf,
    hashes: HashMap<String, String>,
    faces: HashMap<CardId, String>,
}

/// Choose the face for this logical card from the artwork admitted by the collection.
pub fn display_face_id<T>(faces: &HashMap<CardId, T>, identity: CardId) -> CardId {
    if faces.contains_key(&identity) {
        identity
    } else {
        identity.printed_face()
    }
}

impl Collection {
    pub fn open(root: &Path) -> Result<Self, String> {
        let manifest = regular_file(root, "SHA256SUMS").map_err(|error| error.to_string())?;
        let bytes = read_bounded(&manifest, 160_000).map_err(|error| error.to_string())?;
        let matches = |digest| verify_bytes(&bytes, bytes.len() as u64, digest).is_ok();
        let recovered = matches(RECOVERED_MANIFEST_HASH);
        if !recovered && !matches(ORIGINAL_MANIFEST_HASH) && !matches(PHOTOCRAFT_MANIFEST_HASH) {
            return Err(format!(
                "{} differs from the audited card collections.",
                manifest.display()
            ));
        }
        let text = std::str::from_utf8(&bytes).map_err(|error| error.to_string())?;
        let hashes = text
            .lines()
            .filter_map(|line| {
                let (hash, path) = line.split_once("  ")?;
                Some((path.to_owned(), hash.to_owned()))
            })
            .collect();
        let mut collection = Self {
            root: root.to_owned(),
            hashes,
            faces: HashMap::new(),
        };
        let catalog: Vec<CatalogEntry> =
            serde_json::from_slice(&collection.bytes("catalog/cards.json", 300_000)?)
                .map_err(|error| format!("Card catalog: {error}"))?;
        let mut entries: HashMap<_, _> = catalog
            .into_iter()
            .map(|entry| (entry.card_id, entry))
            .collect();
        for row in LEDGER.lines().skip(1) {
            // Only the leading identity and TTS ID are needed. Evidence may be quoted CSV.
            let mut fields = row.splitn(3, ',');
            let identity = fields.next().and_then(|cell| cell.parse::<CardId>().ok());
            let tts_id = fields.next().and_then(|cell| cell.parse::<u32>().ok());
            if let (Some(identity), Some(tts_id)) = (identity, tts_id) {
                // Pin the original roster scans where the ledger has several TTS faces.
                let selected = match identity.serial() {
                    8 => Some(1804),
                    10 => Some(1810),
                    49 => Some(1901),
                    _ => None,
                };
                if selected.is_some_and(|preferred| preferred != tts_id) {
                    continue;
                }
                let entry = entries
                    .remove(&tts_id)
                    .ok_or_else(|| format!("Missing catalog entry for {identity}."))?;
                if entry.back != BACK {
                    return Err(format!("{identity} lacks the original common back."));
                }
                if collection.faces.insert(identity, entry.face).is_some() {
                    return Err(format!(
                        "{identity} has multiple catalog faces and needs an explicit source."
                    ));
                }
            }
        }
        if recovered {
            let recovered_faces: HashMap<CardId, String> =
                serde_json::from_slice(&collection.bytes(RECOVERED_STARTERS, 4000)?)
                    .map_err(|error| format!("Recovered starter catalog: {error}"))?;
            for (identity, face) in recovered_faces {
                let donor = identity.printed_face();
                if identity == donor || !collection.faces.contains_key(&donor) {
                    return Err(format!("Unexpected recovered starter {identity}."));
                }
                if collection.faces.insert(identity, face).is_some() {
                    return Err(format!("Duplicate face source for {identity}."));
                }
            }
        }
        Ok(collection)
    }

    pub fn buy_round_faces(&self) -> Result<HashMap<CardId, DynamicImage>, String> {
        self.public_faces(buy_round_face_ids())
    }

    pub fn expert_series_faces(&self) -> Result<HashMap<CardId, DynamicImage>, String> {
        self.public_faces(buy_round_face_ids().chain((1..=15).chain(46..=60).map(CardId::original)))
    }

    pub fn glove_review_faces(
        &self,
        fixture: GloveReview,
    ) -> Result<HashMap<CardId, DynamicImage>, String> {
        self.public_faces(fixture.face_ids())
    }

    fn public_faces(
        &self,
        identities: impl Iterator<Item = CardId>,
    ) -> Result<HashMap<CardId, DynamicImage>, String> {
        identities
            .map(|identity| display_face_id(&self.faces, identity))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|identity| Ok((identity, self.face(identity)?)))
            .collect()
    }

    pub fn face(&self, identity: CardId) -> Result<DynamicImage, String> {
        let printed = display_face_id(&self.faces, identity);
        let path = self
            .faces
            .get(&printed)
            .ok_or_else(|| format!("Missing card face for {identity} ({printed})."))?;
        self.decode(path)
            .map_err(|error| format!("{identity} ({printed}): {error}"))
    }

    pub fn back(&self) -> Result<DynamicImage, String> {
        self.decode(BACK)
    }

    fn decode(&self, path: &str) -> Result<DynamicImage, String> {
        let bytes = self.bytes(path, 8 * 1024 * 1024)?;
        image::load_from_memory(&bytes).map_err(|error| format!("{path}: {error}"))
    }

    fn bytes(&self, asset: &str, limit: u64) -> Result<Vec<u8>, String> {
        let hash = self
            .hashes
            .get(asset)
            .ok_or_else(|| format!("{asset} lacks an audited digest."))?;
        let path = regular_file(&self.root, asset)
            .map_err(|error| format!("{}: {error}", self.root.join(asset).display()))?;
        let bytes =
            read_bounded(&path, limit).map_err(|error| format!("{}: {error}", path.display()))?;
        verify_bytes(&bytes, bytes.len() as u64, hash)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(bytes)
    }
}
