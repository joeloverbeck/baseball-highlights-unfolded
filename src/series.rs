use crate::{Card, CardId, Player};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    path::{Path, PathBuf},
};
use unfolded_random::SeededRandom;
use unfolded_record_replacement::save;

pub(crate) const SF_IN_PLAY: [u8; 6] = [2, 4, 5, 8, 11, 14];
pub(crate) const NY_IN_PLAY: [u8; 6] = [50, 56, 57, 58, 59, 60];
pub(crate) const OPENING_ROW: [u8; 6] = [61, 62, 63, 64, 65, 66];
pub(crate) const FIRST_REFILL: u8 = 67;
const SHOPPER: Seat = Seat::SanFrancisco;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Seat {
    SanFrancisco,
    NewYork,
}

impl Seat {
    pub fn abbreviation(self) -> &'static str {
        match self {
            Self::SanFrancisco => "SF",
            Self::NewYork => "NY",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::SanFrancisco => 0,
            Self::NewYork => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    Sign { slot: u8, card: Card },
    Demote { card: Card },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Submission {
    pub series: String,
    pub seat: Seat,
    pub revision: u64,
    pub action: Action,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
    WrongSeries,
    StaleDecision,
    WrongSeat,
    WrongDecision,
    OfferChanged,
    Unaffordable,
    Ineligible,
    Save(String),
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::WrongSeries => "The choice belongs to a different series.",
            Self::StaleDecision => "The decision changed. Observe the current choices.",
            Self::WrongSeat => "This seat is observing the other manager’s decision.",
            Self::WrongDecision => "This action is unavailable at the current decision.",
            Self::OfferChanged => "That card does not occupy the selected Buy Row slot.",
            Self::Unaffordable => "SF cannot afford this offer.",
            Self::Ineligible => "Choose an eligible player from SF’s In Play cards.",
            Self::Save(error) => return write!(formatter, "Series save: {error}"),
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for Refusal {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    Signed {
        card: Card,
        slot: u8,
        paid: u8,
        replacement: Card,
    },
    Demoted {
        card: Card,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum Decision {
    Sign { affordable: Vec<Card> },
    Demote { eligible: Vec<Card> },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Offer {
    pub slot: u8,
    pub player: Player,
    pub cost: u8,
    pub balance_after: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Roster {
    pub seat: Seat,
    pub lineup: usize,
    pub in_play: Vec<Player>,
    pub active: usize,
    pub minor_leagues: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand: Option<Vec<Card>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_deck: Option<Card>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Observation {
    pub series: String,
    pub revision: u64,
    pub seat: Seat,
    pub buyer: Seat,
    pub balance: u8,
    pub offers: [Offer; 6],
    pub free_agent_deck: usize,
    pub rosters: [Roster; 2],
    pub decision: Option<Decision>,
    pub events: Vec<Event>,
}

impl Observation {
    pub fn submission(&self, action: Action) -> Submission {
        Submission {
            series: self.series.clone(),
            seat: self.seat,
            revision: self.revision,
            action,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Club {
    // Back of the vector is the top of the face-down pile.
    lineup: Vec<Card>,
    in_play: Vec<Card>,
    minor_leagues: Vec<Card>,
}

impl Club {
    fn observe(&self, seat: Seat, viewer: Seat) -> Roster {
        Roster {
            seat,
            lineup: self.lineup.len(),
            in_play: self.in_play.iter().copied().map(Player::public).collect(),
            active: self.lineup.len() + self.in_play.len(),
            minor_leagues: self.minor_leagues.len(),
            // The reviewed position starts after end-of-game cleanup.
            hand: (seat == viewer).then(Vec::new),
            on_deck: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum Phase {
    Shopping,
    Demotion,
    Complete,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Position {
    series: String,
    private_seed: u64,
    phase: Phase,
    balance: u8,
    row: [Card; 6],
    free_agent_deck: Vec<Card>,
    clubs: [Club; 2],
    events: Vec<Event>,
}

impl Position {
    fn reviewed(series: String, private_seed: u64) -> Self {
        let mut random = SeededRandom::new(private_seed);
        let mut instances: Vec<u8> = (1..=120).collect();
        random.shuffle(&mut instances);
        let physical = |serial| Card {
            identity: CardId::original(serial),
            instance: instances[usize::from(serial - 1)],
        };
        let mut club = |starters: std::ops::RangeInclusive<u8>, in_play: [u8; 6]| {
            let mut lineup: Vec<_> = starters
                .filter(|id| !in_play.contains(id))
                .map(physical)
                .collect();
            random.shuffle(&mut lineup);
            Club {
                lineup,
                in_play: in_play.map(physical).to_vec(),
                minor_leagues: Vec::new(),
            }
        };
        let clubs = [club(1..=15, SF_IN_PLAY), club(46..=60, NY_IN_PLAY)];
        let mut free_agent_deck: Vec<_> = (FIRST_REFILL + 1..=120).map(physical).collect();
        random.shuffle(&mut free_agent_deck);
        free_agent_deck.push(physical(FIRST_REFILL));
        Self {
            series,
            private_seed,
            phase: Phase::Shopping,
            balance: clubs[SHOPPER.index()]
                .in_play
                .iter()
                .map(|&card| Player::public(card).revenue)
                .sum(),
            row: OPENING_ROW.map(physical),
            free_agent_deck,
            clubs,
            events: Vec::new(),
        }
    }

    fn revision(&self) -> u64 {
        // Keep the wire revision exact in JSON clients and distinct across fresh series.
        let prefix = u64::from_str_radix(&self.series[..12], 16).expect("valid series identity");
        prefix * 4 + self.events.len() as u64
    }

    fn observe(&self, seat: Seat) -> Observation {
        let offers = std::array::from_fn(|index| {
            let player = Player::public(self.row[index]);
            let cost = player.cost();
            Offer {
                slot: index as u8 + 1,
                player,
                cost,
                balance_after: self.balance.checked_sub(cost),
            }
        });
        let decision = if seat == SHOPPER {
            match self.phase {
                Phase::Shopping => Some(Decision::Sign {
                    affordable: offers
                        .iter()
                        .filter(|offer| offer.balance_after.is_some())
                        .map(|offer| offer.player.card)
                        .collect(),
                }),
                Phase::Demotion => Some(Decision::Demote {
                    eligible: self.clubs[SHOPPER.index()].in_play.clone(),
                }),
                Phase::Complete => None,
            }
        } else {
            None
        };
        Observation {
            series: self.series.clone(),
            revision: self.revision(),
            seat,
            buyer: SHOPPER,
            balance: self.balance,
            offers,
            free_agent_deck: self.free_agent_deck.len(),
            rosters: [Seat::SanFrancisco, Seat::NewYork]
                .map(|team| self.clubs[team.index()].observe(team, seat)),
            decision,
            events: self.events.clone(),
        }
    }

    fn apply(&mut self, action: Action) -> Result<(), Refusal> {
        let club = &mut self.clubs[SHOPPER.index()];
        match (self.phase, action) {
            (Phase::Shopping, Action::Sign { slot, card }) => {
                let index = usize::from(slot)
                    .checked_sub(1)
                    .ok_or(Refusal::OfferChanged)?;
                let offer = self
                    .row
                    .get_mut(index)
                    .filter(|offer| **offer == card)
                    .ok_or(Refusal::OfferChanged)?;
                let paid = Player::public(card).cost();
                let balance = self
                    .balance
                    .checked_sub(paid)
                    .ok_or(Refusal::Unaffordable)?;
                // The admitted fixture has a replacement ready for its one acquisition.
                let replacement = self
                    .free_agent_deck
                    .pop()
                    .expect("fixture holds its refill");
                self.balance = balance;
                *offer = replacement;
                club.lineup.push(card);
                self.events.push(Event::Signed {
                    card,
                    slot,
                    paid,
                    replacement,
                });
                self.phase = Phase::Demotion;
            }
            (Phase::Demotion, Action::Demote { card }) => {
                let index = club
                    .in_play
                    .iter()
                    .position(|offered| *offered == card)
                    .ok_or(Refusal::Ineligible)?;
                club.minor_leagues.push(club.in_play.remove(index));
                self.events.push(Event::Demoted { card });
                self.phase = Phase::Complete;
            }
            _ => return Err(Refusal::WrongDecision),
        }
        Ok(())
    }
}

struct SeriesFormat;

impl save::Format for SeriesFormat {
    type Payload = Position;
    const LIMIT: u64 = 64 * 1024;

    fn identity(&self, _: &Position) -> save::Identity {
        [
            ("game", "baseball-highlights-unfolded"),
            ("format", "expert-buy-round-1"),
        ]
        .into_iter()
        .collect()
    }

    fn accepts(&self, identity: &save::Identity) -> bool {
        identity.get("game") == Some("baseball-highlights-unfolded")
            && identity.get("format") == Some("expert-buy-round-1")
    }

    fn validate(&self, stored: &Position) -> Result<(), String> {
        if stored.series.len() != 32 || !stored.series.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err("Invalid series identity".into());
        }
        // Verify the zones, event sequence and deck order against the admitted position.
        let mut expected = Position::reviewed(stored.series.clone(), stored.private_seed);
        for event in &stored.events {
            let action = match *event {
                Event::Signed { slot, card, .. } => Action::Sign { slot, card },
                Event::Demoted { card } => Action::Demote { card },
            };
            expected.apply(action).map_err(|error| error.to_string())?;
        }
        (expected == *stored)
            .then_some(())
            .ok_or_else(|| "The saved position disagrees with its decisions.".into())
    }
}

pub struct Game {
    position: Position,
    path: PathBuf,
    baseline: save::Baseline,
}

impl Game {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Refusal> {
        let path = path.as_ref().to_owned();
        let io = |error: save::Error| Refusal::Save(error.to_string());
        let (position, baseline) = match save::load(&SeriesFormat, &path).map_err(io)? {
            Some(record) => record,
            None => {
                let seed = || getrandom::u64().map_err(|error| Refusal::Save(error.to_string()));
                let position =
                    Position::reviewed(format!("{:016x}{:016x}", seed()?, seed()?), seed()?);
                let mut baseline = save::Baseline::default();
                save::write(
                    &SeriesFormat,
                    &path,
                    &mut baseline,
                    &position,
                    save::Write::Create,
                )
                .map_err(io)?;
                (position, baseline)
            }
        };
        Ok(Self {
            position,
            path,
            baseline,
        })
    }

    pub fn observe(&self, seat: Seat) -> Observation {
        self.position.observe(seat)
    }

    pub fn revision(&self) -> u64 {
        self.position.revision()
    }

    /// Refresh a second view of the saved series without acquiring write authority.
    pub fn refresh(&mut self) -> Result<bool, Refusal> {
        let record = save::load(&SeriesFormat, &self.path)
            .map_err(|error| Refusal::Save(error.to_string()))?;
        let (position, baseline) =
            record.ok_or_else(|| Refusal::Save("The saved series is missing.".into()))?;
        if baseline == self.baseline {
            return Ok(false);
        }
        if position.series != self.position.series {
            return Err(Refusal::WrongSeries);
        }
        if !position.events.starts_with(&self.position.events) {
            return Err(Refusal::StaleDecision);
        }
        self.position = position;
        self.baseline = baseline;
        Ok(true)
    }

    pub fn submit(&mut self, choice: Submission) -> Result<(), Refusal> {
        if choice.series != self.position.series {
            return Err(Refusal::WrongSeries);
        }
        if choice.revision != self.position.revision() {
            return Err(Refusal::StaleDecision);
        }
        if choice.seat != SHOPPER {
            return Err(Refusal::WrongSeat);
        }
        let mut next = self.position.clone();
        next.apply(choice.action)?;
        save::write(
            &SeriesFormat,
            &self.path,
            &mut self.baseline,
            &next,
            save::Write::Replace,
        )
        .map_err(|error| Refusal::Save(error.to_string()))?;
        self.position = next;
        Ok(())
    }
}
