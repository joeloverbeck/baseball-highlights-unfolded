//! Admission for the owner-local original collection and its approved proxy faces.
use crate::{
    CardId,
    series::{FIRST_REFILL, NY_IN_PLAY, OPENING_ROW, SF_IN_PLAY},
};
use image::DynamicImage;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use unfolded_assets::{read_bounded, regular_file, verify_bytes};

const MANIFEST_HASH: &str = "c1c528e5ca696ba82c35465b4fb98d57bf830f27376c2cd77582047c54d04ede";
const LEDGER: &str = include_str!("../docs/foundation/card-content.csv");
const BACK: &str = "images/card-backs/backurl-omruwvLU2jpg.jpg";

#[derive(Deserialize)]
struct CatalogEntry {
    card_id: u32,
    face: String,
    back: String,
}

pub struct Collection {
    root: PathBuf,
    hashes: HashMap<String, String>,
    faces: HashMap<CardId, CatalogEntry>,
}

fn hex_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

impl Collection {
    pub fn open(root: &Path) -> Result<Self, String> {
        let manifest = regular_file(root, "SHA256SUMS").map_err(|error| error.to_string())?;
        let bytes = read_bounded(&manifest, 160_000).map_err(|error| error.to_string())?;
        if hex_digest(&bytes) != MANIFEST_HASH {
            return Err(format!(
                "{} differs from the audited original collection.",
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
                let entry = entries
                    .remove(&tts_id)
                    .ok_or_else(|| format!("Missing catalog entry for {identity}."))?;
                if entry.back != BACK {
                    return Err(format!("{identity} lacks the original common back."));
                }
                // Match the reviewed study when more than one catalog face has this ID.
                collection.faces.insert(identity, entry);
            }
        }
        Ok(collection)
    }

    pub fn buy_round_faces(&self) -> Result<HashMap<CardId, DynamicImage>, String> {
        SF_IN_PLAY
            .into_iter()
            .chain(NY_IN_PLAY)
            .chain(OPENING_ROW)
            .chain([FIRST_REFILL])
            .map(CardId::original)
            .map(|identity| Ok((identity.printed_face(), self.face(identity)?)))
            .collect()
    }

    pub fn face(&self, identity: CardId) -> Result<DynamicImage, String> {
        let printed = identity.printed_face();
        let entry = self
            .faces
            .get(&printed)
            .ok_or_else(|| format!("Missing card face for {identity} ({printed})."))?;
        self.decode(&entry.face)
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
        let path = regular_file(&self.root, asset).map_err(|error| error.to_string())?;
        let bytes = read_bounded(&path, limit).map_err(|error| error.to_string())?;
        verify_bytes(&bytes, bytes.len() as u64, hash)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(bytes)
    }
}
