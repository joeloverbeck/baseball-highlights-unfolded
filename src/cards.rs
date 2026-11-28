use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct CardId(u8);

impl CardId {
    pub(crate) const fn original(serial: u8) -> Self {
        Self(serial)
    }

    pub const fn serial(self) -> u8 {
        self.0
    }

    pub fn printed_face(self) -> Self {
        Self(match self.0 {
            3 => 4,
            9 => 10,
            19 => 18,
            24 => 25,
            34 => 33,
            39 => 40,
            48 => 49,
            55 => 54,
            serial => serial,
        })
    }
}

impl FromStr for CardId {
    type Err = &'static str;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let number = text.get(3..).and_then(|value| value.parse::<u8>().ok());
        if let Some(serial @ 1..=120) = number {
            let identity = Self(serial);
            if identity.to_string() == text {
                return Ok(identity);
            }
        }
        Err("Expected an original starter or Free Agent identity")
    }
}

impl TryFrom<String> for CardId {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<CardId> for String {
    fn from(value: CardId) -> Self {
        value.to_string()
    }
}

impl fmt::Display for CardId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = if self.0 <= 60 { "ST" } else { "FA" };
        write!(formatter, "{prefix}-{:03}", self.0)
    }
}

/// The logical identity is separate from the physical instance and printed face.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Card {
    pub identity: CardId,
    pub instance: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Player {
    pub card: Card,
    pub printed_face: CardId,
    pub name: &'static str,
    pub immediate_action: &'static str,
    pub threatened_hits: &'static [&'static str],
    pub revenue: u8,
    pub speed: &'static str,
    pub pinch_hitter: bool,
}

const SINGLE: &[&str] = &["Single"];
const DOUBLE: &[&str] = &["Double"];
const SINGLE_DOUBLE: &[&str] = &["Single", "Double"];
const SINGLES: &[&str] = &["Single", "Single"];
const PICK_OFF: &str = "Pick Off (remove one runner)";
const GLOVE: &str = "Glove (cancel one hit)";
const DOUBLE_PLAY: &str = "Double Play (remove up to two non-fast runners)";

impl Player {
    // Rules facts for the public faces in the reviewed position and its refill.
    // Other cards are present in the physical zones but have no playable rules here.
    pub(crate) fn public(card: Card) -> Self {
        Self::describe(card).expect("the reviewed position only presents known card faces")
    }

    pub fn describe(card: Card) -> Option<Self> {
        let printed_face = card.identity.printed_face();
        let (name, revenue, immediate_action, threatened_hits) = match printed_face.serial() {
            1 => ("Rookie Natural", 0, GLOVE, SINGLE),
            2 | 47 => ("Rookie Cyborg", 2, PICK_OFF, &[][..]),
            4 | 49 => ("Rookie Natural", 2, "", SINGLE),
            5 | 50 => ("Rookie Natural", 1, GLOVE, &[][..]),
            8 | 53 => ("Rookie Robot", 1, "", DOUBLE),
            52 => ("Rookie Robot", 2, "", DOUBLE),
            11 => (
                "Veteran Natural",
                0,
                "Clutch (double with a runner on second or third)",
                SINGLE,
            ),
            14 => ("Veteran Robot", 0, "", &["Home run"][..]),
            56 => ("Veteran Natural", 1, DOUBLE_PLAY, SINGLE),
            57 => (
                "Veteran Cyborg",
                1,
                "Fastball (cancel a Natural's hits)",
                SINGLE,
            ),
            58 => (
                "Veteran Robot",
                2,
                "Quick Eye (single against a Cyborg)",
                DOUBLE,
            ),
            59 => ("Veteran Natural", 0, "", &["Home run"][..]),
            60 => ("Veteran Robot", 2, "", &["Triple"][..]),
            61 => ("Bucky Cano", 2, DOUBLE_PLAY, SINGLES),
            62 => ("Peewee Rizzuto", 1, GLOVE, SINGLE),
            63 => ("Ichiro Matsui", 1, GLOVE, SINGLES),
            64 => (
                "Brooks Nettles",
                1,
                "Clutch (single with a runner on second or third)",
                SINGLES,
            ),
            65 => ("Frankie Foxx", 2, GLOVE, DOUBLE),
            66 => ("Maury Aparicio", 1, DOUBLE_PLAY, SINGLE),
            67 => ("Mickey Maris", 0, GLOVE, &["Single", "Home run"][..]),
            68 => ("Babe Bench", 1, GLOVE, SINGLE_DOUBLE),
            69 => ("Mark Lopes", 3, "", SINGLES),
            82 => ("Willie McGwire", 2, GLOVE, SINGLE_DOUBLE),
            96 => ("Wiffle", 0, "", &["Single", "Single", "Single"][..]),
            _ => return None,
        };
        let speed = match printed_face.serial() {
            52 | 58 | 65 | 68 | 96 => "Slow",
            62 | 63 | 66 | 82 => "Fast",
            _ => "Average",
        };
        let pinch_hitter = matches!(
            printed_face.serial(),
            2 | 5 | 14 | 47 | 50 | 58 | 59 | 62 | 65 | 66
        );
        Some(Self {
            card,
            printed_face,
            name,
            immediate_action,
            threatened_hits,
            revenue,
            speed,
            pinch_hitter,
        })
    }

    pub(crate) fn has_glove(self) -> bool {
        self.immediate_action == GLOVE
    }

    pub(crate) fn cost(self) -> u8 {
        match self.card.identity.serial() {
            61 | 67 => 7,
            62 | 65 => 4,
            63 | 64 => 6,
            66 | 68 | 69 => 5,
            _ => unreachable!("only Free Agents have a signing cost"),
        }
    }
}
