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
pub(crate) const SECOND_REFILL: u8 = 68;
const THIRD_REFILL: u8 = 69;
const SEATS: [Seat; 2] = [Seat::SanFrancisco, Seat::NewYork];

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

    pub fn opponent(self) -> Self {
        match self {
            Self::SanFrancisco => Self::NewYork,
            Self::NewYork => Self::SanFrancisco,
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
    ChooseOrder { first: Seat },
    Sign { slot: u8, card: Card },
    Demote { card: Card },
    Pass,
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
            Self::Unaffordable => "The buyer cannot afford this offer.",
            Self::Ineligible => "Choose an eligible player from the buyer’s In Play cards.",
            Self::Save(error) => return write!(formatter, "Series save: {error}"),
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for Refusal {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    OrderChosen {
        chooser: Seat,
        first: Seat,
    },
    Signed {
        seat: Seat,
        card: Card,
        slot: u8,
        paid: u8,
        replacement: Card,
    },
    Demoted {
        seat: Seat,
        card: Card,
    },
    Passed {
        seat: Seat,
        unspent: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum Decision {
    ChooseOrder,
    /// The buyer may choose any affordable card or finish buying.
    Sign {
        affordable: Vec<Card>,
    },
    Demote {
        eligible: Vec<Card>,
    },
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
    pub dugout: Vec<Player>,
    pub active: usize,
    pub minor_leagues: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand: Option<Vec<Card>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_deck: Option<Card>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Contribution {
    pub card: Card,
    pub amount: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Revenue {
    pub seat: Seat,
    pub contributions: Vec<Contribution>,
    pub total: u8,
}

impl Revenue {
    fn earned(seat: Seat, cards: &[Card]) -> Self {
        let contributions: Vec<_> = cards
            .iter()
            .map(|&card| Contribution {
                card,
                amount: Player::public(card).revenue,
            })
            .collect();
        let total = contributions.iter().map(|entry| entry.amount).sum();
        Self {
            seat,
            contributions,
            total,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Observation {
    pub series: String,
    pub revision: u64,
    pub seat: Seat,
    pub order_chooser: Seat,
    pub first_buyer: Option<Seat>,
    pub acting_manager: Option<Seat>,
    pub buyer: Option<Seat>,
    pub demotion_pending: bool,
    pub revenue: [Revenue; 2],
    pub balances: [u8; 2],
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
    #[serde(default)]
    dugout: Vec<Card>,
    minor_leagues: Vec<Card>,
}

impl Club {
    fn observe(&self, seat: Seat, viewer: Seat) -> Roster {
        Roster {
            seat,
            lineup: self.lineup.len(),
            in_play: self.in_play.iter().copied().map(Player::public).collect(),
            dugout: self.dugout.iter().copied().map(Player::public).collect(),
            active: self.lineup.len() + self.in_play.len() + self.dugout.len(),
            minor_leagues: self.minor_leagues.len(),
            // The reviewed position starts after end-of-game cleanup.
            hand: (seat == viewer).then(Vec::new),
            on_deck: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum Phase {
    Order,
    Shopping,
    Demotion,
    Complete,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BuyRoundFixture {
    Reviewed,
    RevenueTie,
    RepeatPurchase,
}

impl BuyRoundFixture {
    const ALL: [Self; 3] = [Self::Reviewed, Self::RevenueTie, Self::RepeatPurchase];

    fn ny_in_play(self) -> [u8; 6] {
        match self {
            Self::Reviewed => NY_IN_PLAY,
            Self::RevenueTie => [50, 53, 56, 57, 58, 59],
            Self::RepeatPurchase => [47, 49, 50, 52, 58, 60],
        }
    }

    fn winner(self) -> Seat {
        match self {
            Self::RevenueTie => Seat::SanFrancisco,
            Self::Reviewed | Self::RepeatPurchase => Seat::NewYork,
        }
    }

    fn refills(self) -> &'static [u8] {
        match self {
            Self::Reviewed | Self::RevenueTie => &[FIRST_REFILL, SECOND_REFILL],
            Self::RepeatPurchase => &[FIRST_REFILL, SECOND_REFILL, THIRD_REFILL],
        }
    }
}

pub(crate) fn buy_round_face_ids() -> impl Iterator<Item = CardId> {
    SF_IN_PLAY
        .into_iter()
        .chain(OPENING_ROW)
        .chain(BuyRoundFixture::ALL.into_iter().flat_map(|fixture| {
            fixture
                .ny_in_play()
                .into_iter()
                .chain(fixture.refills().iter().copied())
        }))
        .map(CardId::original)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Position {
    series: String,
    private_seed: u64,
    fixture: BuyRoundFixture,
    phase: Phase,
    first_buyer: Option<Seat>,
    buyer: Option<Seat>,
    winner: Seat,
    revenue: [Revenue; 2],
    balances: [u8; 2],
    row: [Card; 6],
    free_agent_deck: Vec<Card>,
    clubs: [Club; 2],
    events: Vec<Event>,
}

impl Position {
    fn new(series: String, private_seed: u64, fixture: BuyRoundFixture) -> Self {
        Self::setup(series, private_seed, fixture, fixture.refills())
    }

    fn setup(series: String, private_seed: u64, fixture: BuyRoundFixture, refills: &[u8]) -> Self {
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
                dugout: Vec::new(),
                minor_leagues: Vec::new(),
            }
        };
        let clubs = [
            club(1..=15, SF_IN_PLAY),
            club(46..=60, fixture.ny_in_play()),
        ];
        // Start with the original seeded tail so the reviewed continuation only
        // moves the additional fixed refills, preserving all other card positions
        // relative to one another and every instance identifier.
        let mut free_agent_deck: Vec<_> = (FIRST_REFILL + 1..=120).map(physical).collect();
        random.shuffle(&mut free_agent_deck);
        free_agent_deck.retain(|card| !refills.contains(&card.identity.serial()));
        free_agent_deck.extend(refills.iter().rev().copied().map(physical));
        let revenue = SEATS.map(|seat| Revenue::earned(seat, &clubs[seat.index()].in_play));
        let balances = revenue.each_ref().map(|income| income.total);
        Self {
            series,
            private_seed,
            fixture,
            phase: Phase::Order,
            first_buyer: None,
            buyer: None,
            winner: fixture.winner(),
            revenue,
            balances,
            row: OPENING_ROW.map(physical),
            free_agent_deck,
            clubs,
            events: Vec::new(),
        }
    }

    fn order_chooser(&self) -> Seat {
        use std::cmp::Ordering;
        match self.revenue[0].total.cmp(&self.revenue[1].total) {
            Ordering::Less => Seat::SanFrancisco,
            Ordering::Greater => Seat::NewYork,
            Ordering::Equal => self.winner.opponent(),
        }
    }

    fn acting_manager(&self) -> Option<Seat> {
        match self.phase {
            Phase::Order => Some(self.order_chooser()),
            Phase::Shopping | Phase::Demotion => self.buyer,
            Phase::Complete => None,
        }
    }

    fn revision(&self) -> u64 {
        // Leave room for this round's choices while keeping revisions exact in JSON clients.
        let prefix = u64::from_str_radix(&self.series[..12], 16).expect("valid series identity");
        prefix * 16 + self.events.len() as u64
    }

    fn observe(&self, seat: Seat) -> Observation {
        let buyer_balance = self.buyer.map(|buyer| self.balances[buyer.index()]);
        let offers = std::array::from_fn(|index| {
            let player = Player::public(self.row[index]);
            let cost = player.cost();
            Offer {
                slot: index as u8 + 1,
                player,
                cost,
                balance_after: buyer_balance.and_then(|balance| balance.checked_sub(cost)),
            }
        });
        let acting_manager = self.acting_manager();
        let decision = if acting_manager == Some(seat) {
            match self.phase {
                Phase::Order => Some(Decision::ChooseOrder),
                Phase::Shopping => Some(Decision::Sign {
                    affordable: offers
                        .iter()
                        .filter(|offer| offer.balance_after.is_some())
                        .map(|offer| offer.player.card)
                        .collect(),
                }),
                Phase::Demotion => Some(Decision::Demote {
                    eligible: self.clubs[seat.index()].in_play.clone(),
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
            order_chooser: self.order_chooser(),
            first_buyer: self.first_buyer,
            acting_manager,
            buyer: self.buyer,
            demotion_pending: self.phase == Phase::Demotion,
            revenue: self.revenue.clone(),
            balances: self.balances,
            balance: buyer_balance.unwrap_or(0),
            offers,
            free_agent_deck: self.free_agent_deck.len(),
            rosters: SEATS.map(|team| self.clubs[team.index()].observe(team, seat)),
            decision,
            events: self.events.clone(),
        }
    }

    fn apply(&mut self, action: Action) -> Result<(), Refusal> {
        let seat = self.acting_manager().ok_or(Refusal::WrongDecision)?;
        let club = &mut self.clubs[seat.index()];
        let balance = &mut self.balances[seat.index()];
        match (self.phase, action) {
            (Phase::Order, Action::ChooseOrder { first }) => {
                self.events.push(Event::OrderChosen {
                    chooser: seat,
                    first,
                });
                self.first_buyer = Some(first);
                self.buyer = Some(first);
                self.phase = Phase::Shopping;
            }
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
                let remaining = balance.checked_sub(paid).ok_or(Refusal::Unaffordable)?;
                let replacement = self.free_agent_deck.pop().ok_or(Refusal::OfferChanged)?;
                *balance = remaining;
                *offer = replacement;
                club.lineup.push(card);
                self.events.push(Event::Signed {
                    seat,
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
                self.events.push(Event::Demoted { seat, card });
                self.phase = Phase::Shopping;
            }
            (Phase::Shopping, Action::Pass) => {
                self.events.push(Event::Passed {
                    seat,
                    unspent: std::mem::take(balance),
                });
                if self.first_buyer == Some(seat) {
                    self.buyer = Some(seat.opponent());
                } else {
                    for club in &mut self.clubs {
                        club.dugout.append(&mut club.in_play);
                    }
                    self.buyer = None;
                    self.phase = Phase::Complete;
                }
            }
            _ => return Err(Refusal::WrongDecision),
        }
        Ok(())
    }
}

impl Event {
    fn action(self) -> Action {
        match self {
            Self::OrderChosen { first, .. } => Action::ChooseOrder { first },
            Self::Signed { slot, card, .. } => Action::Sign { slot, card },
            Self::Demoted { card, .. } => Action::Demote { card },
            Self::Passed { .. } => Action::Pass,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum LegacyEvent {
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

impl LegacyEvent {
    fn with_seat(self, seat: Seat) -> Event {
        match self {
            Self::Signed {
                card,
                slot,
                paid,
                replacement,
            } => Event::Signed {
                seat,
                card,
                slot,
                paid,
                replacement,
            },
            Self::Demoted { card } => Event::Demoted { seat, card },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct LegacyPosition {
    series: String,
    private_seed: u64,
    phase: Phase,
    balance: u8,
    row: [Card; 6],
    free_agent_deck: Vec<Card>,
    clubs: [Club; 2],
    events: Vec<LegacyEvent>,
}

impl LegacyPosition {
    fn restore(&self) -> Result<Position, String> {
        let expected_phase = match self.events.as_slice() {
            [] => Phase::Shopping,
            [LegacyEvent::Signed { .. }] => Phase::Demotion,
            [LegacyEvent::Signed { .. }, LegacyEvent::Demoted { .. }] => Phase::Complete,
            _ => return Err("Invalid legacy Buy Round event sequence.".into()),
        };
        let seat = Seat::SanFrancisco;
        let restore = |refills: &[u8]| {
            let mut position = Position::setup(
                self.series.clone(),
                self.private_seed,
                BuyRoundFixture::Reviewed,
                refills,
            );
            if !self.events.is_empty() {
                // A saved acquisition records SF buying first under the original slice.
                position.apply(Action::ChooseOrder { first: seat })?;
                for event in &self.events {
                    position.apply(event.with_seat(seat).action())?;
                }
            }
            Ok::<_, Refusal>(position)
        };
        let original = restore(&[FIRST_REFILL]).map_err(|error| error.to_string())?;
        let events = self.events.iter().map(|event| event.with_seat(seat));
        if self.phase != expected_phase
            || self.balance != original.balances[seat.index()]
            || self.row != original.row
            || self.free_agent_deck != original.free_agent_deck
            || self.clubs != original.clubs
            || !original.events.iter().copied().skip(1).eq(events)
        {
            return Err("The legacy Buy Round save disagrees with its decisions.".into());
        }
        restore(BuyRoundFixture::Reviewed.refills()).map_err(|error| error.to_string())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum SavedPosition {
    Current(Position),
    Legacy(LegacyPosition),
}

impl SavedPosition {
    fn into_position(self) -> Result<Position, Refusal> {
        match self {
            Self::Current(position) => Ok(position),
            Self::Legacy(position) => position.restore().map_err(Refusal::Save),
        }
    }
}

struct SeriesFormat;

impl save::Format for SeriesFormat {
    type Payload = SavedPosition;
    const LIMIT: u64 = 64 * 1024;

    fn identity(&self, payload: &SavedPosition) -> save::Identity {
        let format = match payload {
            SavedPosition::Current(_) => "expert-buy-round-2",
            SavedPosition::Legacy(_) => "expert-buy-round-1",
        };
        [("game", "baseball-highlights-unfolded"), ("format", format)]
            .into_iter()
            .collect()
    }

    fn accepts(&self, identity: &save::Identity) -> bool {
        identity.get("game") == Some("baseball-highlights-unfolded")
            && matches!(
                identity.get("format"),
                Some("expert-buy-round-1" | "expert-buy-round-2")
            )
    }

    fn validate(&self, payload: &SavedPosition) -> Result<(), String> {
        let series = match payload {
            SavedPosition::Current(stored) => &stored.series,
            SavedPosition::Legacy(stored) => &stored.series,
        };
        if series.len() != 32 || !series.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("Invalid series identity".into());
        }
        let stored = match payload {
            SavedPosition::Current(stored) => stored,
            SavedPosition::Legacy(stored) => return stored.restore().map(|_| ()),
        };
        // Verify the zones, event sequence and deck order against the admitted position.
        let mut expected =
            Position::new(stored.series.clone(), stored.private_seed, stored.fixture);
        for event in &stored.events {
            expected
                .apply(event.action())
                .map_err(|error| error.to_string())?;
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
        Self::open_buy_round_fixture(path, BuyRoundFixture::Reviewed)
    }

    /// Start a fresh fixture at the end of its completed mini-game, or reopen its save.
    pub fn open_buy_round_fixture(
        path: impl AsRef<Path>,
        fixture: BuyRoundFixture,
    ) -> Result<Self, Refusal> {
        let path = path.as_ref().to_owned();
        let io = |error: save::Error| Refusal::Save(error.to_string());
        let (position, baseline) = match save::load(&SeriesFormat, &path).map_err(io)? {
            Some((stored, baseline)) => (stored.into_position()?, baseline),
            None => {
                let seed = || getrandom::u64().map_err(|error| Refusal::Save(error.to_string()));
                let position = Position::new(
                    format!("{:016x}{:016x}", seed()?, seed()?),
                    seed()?,
                    fixture,
                );
                let mut baseline = save::Baseline::default();
                save::write(
                    &SeriesFormat,
                    &path,
                    &mut baseline,
                    &SavedPosition::Current(position.clone()),
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
        let (stored, baseline) =
            record.ok_or_else(|| Refusal::Save("The saved series is missing.".into()))?;
        if baseline == self.baseline {
            return Ok(false);
        }
        let position = stored.into_position()?;
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
        let acting_manager = self
            .position
            .acting_manager()
            .ok_or(Refusal::WrongDecision)?;
        if choice.seat != acting_manager {
            return Err(Refusal::WrongSeat);
        }
        let mut next = self.position.clone();
        next.apply(choice.action)?;
        save::write(
            &SeriesFormat,
            &self.path,
            &mut self.baseline,
            &SavedPosition::Current(next.clone()),
            save::Write::Replace,
        )
        .map_err(|error| Refusal::Save(error.to_string()))?;
        self.position = next;
        Ok(())
    }
}
