use crate::{
    Card, CardId, FieldView, GlovePreview, GloveReview, Hit, HitBox, Player, Threat, field::Field,
};
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
const FIRST_HOME: Seat = Seat::SanFrancisco;
const OPENING_PLAYS: usize = 3;

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

    pub(crate) fn index(self) -> usize {
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
    StartGame,
    PlayCard { card: Card },
    CancelHit { target: HitBox },
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
    CardNotInHand,
    HitChanged,
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
            Self::CardNotInHand => "Choose a card from this manager’s hand.",
            Self::HitChanged => "That hit box is unavailable on the opposing play.",
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
    GameStarted {
        home: Seat,
    },
    CardPlayed {
        seat: Seat,
        card: Card,
    },
    FastballCancelled {
        seat: Seat,
    },
    ImmediateHit {
        seat: Seat,
        card: Card,
        hit: Hit,
        runs: u16,
    },
    GloveCancelled {
        seat: Seat,
        threat: Threat,
        chosen: bool,
    },
    HitResolved {
        seat: Seat,
        threat: Threat,
        runs: u16,
    },
    ThreatsPlaced {
        seat: Seat,
        card: Card,
        count: u8,
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
    StartGame,
    PlayCard {
        hand: Vec<Player>,
    },
    Glove {
        targets: Vec<GlovePreview>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standing: Option<Standing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<FieldView>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Standing {
    pub game: u8,
    pub home: Seat,
    /// SF and NY games won, in that order.
    pub wins: [u8; 2],
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    hand: Vec<Card>,
}

impl Club {
    fn observe(&self, seat: Seat, viewer: Seat) -> Roster {
        Roster {
            seat,
            lineup: self.lineup.len(),
            in_play: self.in_play.iter().copied().map(Player::public).collect(),
            dugout: self.dugout.iter().copied().map(Player::public).collect(),
            active: self.lineup.len() + self.in_play.len() + self.dugout.len() + self.hand.len(),
            minor_leagues: self.minor_leagues.len(),
            hand: (seat == viewer).then(|| self.hand.clone()),
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    field_review: Option<FieldReview>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    opening: Option<Field>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct FieldReview {
    fixture: GloveReview,
    field: Field,
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
                hand: Vec::new(),
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
            field_review: None,
            opening: None,
        }
    }

    fn new_glove_review(series: String, private_seed: u64, fixture: GloveReview) -> Self {
        let mut position = Self::new(series, private_seed, BuyRoundFixture::Reviewed);
        position.phase = Phase::Complete;
        position.balances = [0; 2];
        for club in &mut position.clubs {
            club.dugout.append(&mut club.in_play);
        }
        let (defender, opposing) = fixture.cards();
        let runner = fixture
            .runner()
            .map(|serial| position.put_review_card_in_play(Seat::NewYork, serial));
        let defender = position.put_review_card_in_play(Seat::SanFrancisco, defender);
        let opposing = position.put_review_card_in_play(Seat::NewYork, opposing);
        position.field_review = Some(FieldReview {
            fixture,
            field: Field::reviewed(defender, opposing, runner, &mut position.events),
        });
        position
    }

    fn put_review_card_in_play(&mut self, seat: Seat, serial: u8) -> Card {
        let club = &mut self.clubs[seat.index()];
        let identity = CardId::original(serial);
        let card = if let Some(index) = club
            .lineup
            .iter()
            .position(|card| card.identity == identity)
        {
            club.lineup.remove(index)
        } else if let Some(index) = club
            .dugout
            .iter()
            .position(|card| card.identity == identity)
        {
            club.dugout.remove(index)
        } else {
            let index = self
                .free_agent_deck
                .iter()
                .position(|card| card.identity == identity)
                .expect("the reviewed Free Agent is in the original deck");
            let demoted = club.dugout.remove(0);
            club.minor_leagues.push(demoted);
            self.free_agent_deck.remove(index)
        };
        club.in_play.push(card);
        card
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
        if let Some(review) = &self.field_review {
            return review.field.manager();
        }
        if let Some(field) = &self.opening {
            return field
                .manager()
                .or_else(|| (self.opening_plays() < OPENING_PLAYS).then(|| field.acting()));
        }
        match self.phase {
            Phase::Order => Some(self.order_chooser()),
            Phase::Shopping | Phase::Demotion => self.buyer,
            Phase::Complete => Some(FIRST_HOME),
        }
    }

    fn opening_plays(&self) -> usize {
        self.clubs.iter().map(|club| club.in_play.len()).sum()
    }

    fn revision(&self) -> u64 {
        // Keep revisions exact in JSON clients. Submissions also carry the series identity.
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
        let decision = if let Some(review) = &self.field_review {
            review.field.decision(seat)
        } else if let Some(field) = &self.opening {
            if field.manager().is_some() {
                field.decision(seat)
            } else if acting_manager == Some(seat) {
                Some(Decision::PlayCard {
                    hand: self.clubs[seat.index()]
                        .hand
                        .iter()
                        .copied()
                        .map(Player::public)
                        .collect(),
                })
            } else {
                None
            }
        } else if acting_manager == Some(seat) {
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
                Phase::Complete => Some(Decision::StartGame),
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
            standing: self.field_review.is_none().then(|| Standing {
                game: if self.opening.is_some() { 2 } else { 1 },
                home: if self.opening.is_some() {
                    FIRST_HOME.opponent()
                } else {
                    FIRST_HOME
                },
                wins: SEATS.map(|seat| u8::from(self.winner == seat)),
            }),
            field: self
                .field_review
                .as_ref()
                .map(|review| &review.field)
                .or(self.opening.as_ref())
                .map(Field::view),
        }
    }

    fn apply(&mut self, action: Action) -> Result<(), Refusal> {
        if let Some(review) = &mut self.field_review {
            return match action {
                Action::CancelHit { target } => review.field.cancel_hit(target, &mut self.events),
                _ => Err(Refusal::WrongDecision),
            };
        }
        let seat = self.acting_manager().ok_or(Refusal::WrongDecision)?;
        if let Some(field) = &mut self.opening {
            return match action {
                Action::CancelHit { target } => field.cancel_hit(target, &mut self.events),
                Action::PlayCard { card } if field.manager().is_none() => {
                    let club = &mut self.clubs[seat.index()];
                    let index = club
                        .hand
                        .iter()
                        .position(|candidate| *candidate == card)
                        .ok_or(Refusal::CardNotInHand)?;
                    club.in_play.push(club.hand.remove(index));
                    self.events.push(Event::CardPlayed { seat, card });
                    field.play_card(card, &mut self.events)
                }
                _ => Err(Refusal::WrongDecision),
            };
        }
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
            (Phase::Complete, Action::StartGame) => {
                let home = FIRST_HOME.opponent();
                for club in &mut self.clubs {
                    // The reviewed Buy Round always leaves at least nine in each Lineup.
                    club.hand = (0..6)
                        .map(|_| club.lineup.pop().expect("opening Lineup"))
                        .collect();
                }
                self.opening = Some(Field::opening(home));
                self.events.push(Event::GameStarted { home });
            }
            _ => return Err(Refusal::WrongDecision),
        }
        Ok(())
    }
}

impl Event {
    fn action(self) -> Option<Action> {
        match self {
            Self::OrderChosen { first, .. } => Some(Action::ChooseOrder { first }),
            Self::Signed { slot, card, .. } => Some(Action::Sign { slot, card }),
            Self::Demoted { card, .. } => Some(Action::Demote { card }),
            Self::Passed { .. } => Some(Action::Pass),
            Self::GameStarted { .. } => Some(Action::StartGame),
            Self::CardPlayed { card, .. } => Some(Action::PlayCard { card }),
            Self::GloveCancelled {
                threat,
                chosen: true,
                ..
            } => Some(Action::CancelHit {
                target: threat.target,
            }),
            Self::GloveCancelled { chosen: false, .. }
            | Self::FastballCancelled { .. }
            | Self::ImmediateHit { .. }
            | Self::HitResolved { .. }
            | Self::ThreatsPlaced { .. } => None,
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
                    let action = event
                        .with_seat(seat)
                        .action()
                        .expect("legacy events are choices");
                    position.apply(action)?;
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
            SavedPosition::Current(position) if position.field_review.is_some() => "glove-review-1",
            SavedPosition::Current(position) if position.opening.is_some() => "expert-series-1",
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
                Some(
                    "expert-buy-round-1"
                        | "expert-buy-round-2"
                        | "glove-review-1"
                        | "expert-series-1"
                )
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
        let mut expected = match &stored.field_review {
            Some(review) => Position::new_glove_review(
                stored.series.clone(),
                stored.private_seed,
                review.fixture,
            ),
            None => Position::new(stored.series.clone(), stored.private_seed, stored.fixture),
        };
        for action in stored.events.iter().copied().filter_map(Event::action) {
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
        Self::open_buy_round_fixture(path, BuyRoundFixture::Reviewed)
    }

    /// Start a fresh fixture at the end of its completed mini-game, or reopen its save.
    pub fn open_buy_round_fixture(
        path: impl AsRef<Path>,
        fixture: BuyRoundFixture,
    ) -> Result<Self, Refusal> {
        Self::open_position(path.as_ref(), |series, seed| {
            Position::new(series, seed, fixture)
        })
    }

    /// Start or resume the bounded Glove review using a separate save.
    pub fn open_glove_review(
        path: impl AsRef<Path>,
        fixture: GloveReview,
    ) -> Result<Self, Refusal> {
        let game = Self::open_position(path.as_ref(), |series, seed| {
            Position::new_glove_review(series, seed, fixture)
        })?;
        if game
            .position
            .field_review
            .as_ref()
            .map(|review| review.fixture)
            != Some(fixture)
        {
            return Err(Refusal::Save(
                "This save belongs to a different position.".into(),
            ));
        }
        Ok(game)
    }

    fn open_position(
        path: &Path,
        create: impl FnOnce(String, u64) -> Position,
    ) -> Result<Self, Refusal> {
        let path = path.to_owned();
        let io = |error: save::Error| Refusal::Save(error.to_string());
        let (position, baseline) = match save::load(&SeriesFormat, &path).map_err(io)? {
            Some((stored, baseline)) => (stored.into_position()?, baseline),
            None => {
                let seed = || getrandom::u64().map_err(|error| Refusal::Save(error.to_string()));
                let position = create(format!("{:016x}{:016x}", seed()?, seed()?), seed()?);
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

    pub fn glove_review(&self) -> Option<GloveReview> {
        self.position
            .field_review
            .as_ref()
            .map(|review| review.fixture)
    }

    pub fn revision(&self) -> u64 {
        self.position.revision()
    }

    pub fn has_field(&self) -> bool {
        self.position.field_review.is_some() || self.position.opening.is_some()
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
        if position.series != self.position.series
            || position.field_review.as_ref().map(|review| review.fixture) != self.glove_review()
        {
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
