use crate::{Card, Decision, Event, Player, Refusal, Seat};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GloveReview {
    Maris,
    Lone,
    NoTarget,
    Rulebook,
}

impl GloveReview {
    pub(crate) fn cards(self) -> (u8, u8) {
        match self {
            Self::Maris => (1, 67),
            Self::Lone => (1, 56),
            Self::NoTarget => (1, 50),
            Self::Rulebook => (82, 96),
        }
    }

    pub(crate) fn face_ids(self) -> impl Iterator<Item = crate::CardId> {
        let (defender, opposing) = self.cards();
        [defender, opposing]
            .into_iter()
            .chain(self.runner())
            .map(crate::CardId::original)
    }

    pub(crate) fn runner(self) -> Option<u8> {
        (self == Self::Maris).then_some(57)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerSpeed {
    Slow,
    Average,
    Fast,
}

impl RunnerSpeed {
    fn of(player: Player) -> Self {
        match player.speed {
            "Slow" => Self::Slow,
            "Average" => Self::Average,
            "Fast" => Self::Fast,
            _ => unreachable!("unknown printed speed"),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Slow => "Slow",
            Self::Average => "Average",
            Self::Fast => "Fast",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Hit {
    Single,
    Double,
    Triple,
    HomeRun,
}

impl Hit {
    fn printed(label: &str) -> Self {
        match label {
            "Single" => Self::Single,
            "Double" => Self::Double,
            "Triple" => Self::Triple,
            "Home run" => Self::HomeRun,
            _ => unreachable!("unknown printed hit"),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Single => "Single",
            Self::Double => "Double",
            Self::Triple => "Triple",
            Self::HomeRun => "Home run",
        }
    }

    fn distance(self) -> usize {
        match self {
            Self::Single => 1,
            Self::Double => 2,
            Self::Triple => 3,
            Self::HomeRun => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HitBox {
    pub card: Card,
    /// The one-based position on the source card.
    pub number: u8,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Runner {
    pub card: Card,
    pub speed: RunnerSpeed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Threat {
    pub target: HitBox,
    pub hit: Hit,
    pub batter: Runner,
}

impl Threat {
    fn on(card: Card) -> Vec<Self> {
        let player = Player::public(card);
        let batter = Runner {
            card,
            speed: RunnerSpeed::of(player),
        };
        player
            .threatened_hits
            .iter()
            .enumerate()
            .map(|(index, hit)| Self {
                target: HitBox {
                    card,
                    number: index as u8 + 1,
                },
                hit: Hit::printed(hit),
                batter,
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Diamond {
    pub score: u16,
    /// First, second and third base in that order.
    pub bases: [Option<Runner>; 3],
}

impl Diamond {
    fn resolve(&mut self, threat: Threat) -> u16 {
        let before = self.score;
        let mut blocker = None;
        let distance = threat.hit.distance();
        for base in (0..self.bases.len()).rev() {
            let Some(runner) = self.bases[base].take() else {
                continue;
            };
            let extra = usize::from(
                runner.speed == RunnerSpeed::Fast
                    || (runner.speed == RunnerSpeed::Average
                        && base == 1
                        && threat.hit == Hit::Single),
            );
            let destination = (base + distance + extra).min(blocker.map_or(3, |next| next - 1));
            if destination == 3 {
                self.score += 1;
            } else {
                self.bases[destination] = Some(runner);
                blocker = Some(destination);
            }
        }
        if threat.hit == Hit::HomeRun {
            self.score += 1;
        } else {
            self.bases[distance - 1] = Some(threat.batter);
        }
        self.score - before
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Play {
    defender: Card,
    opposing: Card,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct FieldPlay {
    pub defender: Player,
    pub batter: Player,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FieldView {
    pub home: Seat,
    pub acting: Seat,
    pub batting: Seat,
    /// The SF diamond followed by the NY diamond.
    pub diamonds: [Diamond; 2],
    pub play: FieldPlay,
    pub threatened: Vec<Threat>,
    pub glove_pending: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GlovePreview {
    pub target: HitBox,
    pub cancelled: Hit,
    pub surviving: Vec<Threat>,
    /// The SF diamond followed by the NY diamond after resolving the threats.
    pub diamonds: [Diamond; 2],
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Field {
    home: Seat,
    acting: Seat,
    diamonds: [Diamond; 2],
    threatened: Vec<Threat>,
    play: Play,
    glove_pending: bool,
}

impl Field {
    pub(crate) fn reviewed(
        defender: Card,
        opposing: Card,
        runner: Option<Card>,
        events: &mut Vec<Event>,
    ) -> Self {
        let mut field = Self {
            home: Seat::NewYork,
            acting: Seat::SanFrancisco,
            diamonds: [Diamond::default(); 2],
            threatened: Threat::on(opposing),
            play: Play { defender, opposing },
            glove_pending: false,
        };
        if let Some(card) = runner {
            field.diamonds[Seat::SanFrancisco.index()].score = 1;
            field.diamonds[Seat::NewYork.index()] = Diamond {
                score: 2,
                bases: [
                    None,
                    Some(Runner {
                        card,
                        speed: RunnerSpeed::of(Player::public(card)),
                    }),
                    None,
                ],
            };
        }
        assert!(Player::public(defender).has_glove());
        match field.threatened.as_slice() {
            [] => field.resolve(events),
            [only] => {
                field
                    .cancel(only.target, false, events)
                    .expect("the only hit box is present");
                field.resolve(events);
            }
            _ => field.glove_pending = true,
        }
        field
    }

    pub(crate) fn manager(&self) -> Option<Seat> {
        self.glove_pending.then_some(self.acting)
    }

    pub(crate) fn view(&self) -> FieldView {
        FieldView {
            home: self.home,
            acting: self.acting,
            batting: self.acting.opponent(),
            diamonds: self.diamonds,
            play: FieldPlay {
                defender: Player::public(self.play.defender),
                batter: Player::public(self.play.opposing),
            },
            threatened: self.threatened.clone(),
            glove_pending: self.glove_pending,
        }
    }

    pub(crate) fn decision(&self, viewer: Seat) -> Option<Decision> {
        (self.glove_pending && viewer == self.acting).then(|| Decision::Glove {
            targets: self
                .threatened
                .iter()
                .map(|threat| {
                    let surviving: Vec<_> = self
                        .threatened
                        .iter()
                        .copied()
                        .filter(|other| other.target != threat.target)
                        .collect();
                    let mut diamonds = self.diamonds;
                    let diamond = &mut diamonds[self.acting.opponent().index()];
                    for &hit in &surviving {
                        diamond.resolve(hit);
                    }
                    GlovePreview {
                        target: threat.target,
                        cancelled: threat.hit,
                        surviving,
                        diamonds,
                    }
                })
                .collect(),
        })
    }

    pub(crate) fn cancel_hit(
        &mut self,
        target: HitBox,
        events: &mut Vec<Event>,
    ) -> Result<(), Refusal> {
        if !self.glove_pending {
            return Err(Refusal::WrongDecision);
        }
        self.cancel(target, true, events)?;
        self.resolve(events);
        Ok(())
    }

    fn cancel(
        &mut self,
        target: HitBox,
        chosen: bool,
        events: &mut Vec<Event>,
    ) -> Result<(), Refusal> {
        let index = self
            .threatened
            .iter()
            .position(|threat| threat.target == target)
            .ok_or(Refusal::HitChanged)?;
        let threat = self.threatened.remove(index);
        events.push(Event::GloveCancelled {
            seat: self.acting,
            threat,
            chosen,
        });
        Ok(())
    }

    fn resolve(&mut self, events: &mut Vec<Event>) {
        let batting = self.acting.opponent();
        let diamond = &mut self.diamonds[batting.index()];
        for threat in self.threatened.drain(..) {
            let runs = diamond.resolve(threat);
            events.push(Event::HitResolved {
                seat: batting,
                threat,
                runs,
            });
        }
        let card = self.play.defender;
        self.threatened = Threat::on(card);
        events.push(Event::ThreatsPlaced {
            seat: self.acting,
            card,
            count: self.threatened.len() as u8,
        });
        self.glove_pending = false;
        self.acting = batting;
    }
}
