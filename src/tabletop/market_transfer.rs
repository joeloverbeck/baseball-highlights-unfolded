//! Play committed card movements through the visible Press Box zones.

use super::{
    Prints, Session,
    controls::Board,
    market::{Layout, OfferFace, place},
};
use baseball_highlights_unfolded::{Card, Event, Observation, Seat};
use bevy::{picking::prelude::Pickable, prelude::*};
use std::f32::consts::PI;
use unfolded_tabletop::card_motion::{CardMove, Departure, Lift, Turn};

const CARD_ASPECT: f32 = 1.393;
const TRAVEL_SECONDS: f32 = 0.65;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Zone {
    Lineup(Seat),
    Minors(Seat),
    Offer(Card),
}

#[derive(Component)]
pub(super) struct PileTop {
    pub(super) zone: Zone,
    pub(super) count: usize,
}

struct Flight {
    entity: Entity,
    destination: Zone,
    player: Card,
    path: CardMove,
    elapsed: f32,
}

#[derive(Resource, Default)]
pub(super) struct Playback {
    previous: Option<Observation>,
    board: Option<Entity>,
    flights: Vec<Flight>,
}

impl Playback {
    pub(super) fn moving(&self) -> bool {
        !self.flights.is_empty()
    }
}

fn endpoint(rect: Rect) -> Vec3 {
    let center = rect.center();
    Vec3::new(center.x, -center.y, rect.width())
}

fn draw_flight(
    commands: &mut Commands,
    board: Entity,
    player: Card,
    from: Rect,
    to: Rect,
    zone: Zone,
    prints: &Prints,
    scale: f32,
) -> Flight {
    let face_up = matches!(zone, Zone::Offer(_));
    let mut path = CardMove::new(
        endpoint(from),
        Transform::from_translation(endpoint(to)),
        TRAVEL_SECONDS,
    )
    .lifted(Lift::arc(29. * scale))
    .turned(Turn {
        axis: Vec3::Y,
        from: if face_up { PI } else { 0. },
        to: if face_up { 0. } else { PI },
        begins: 0.22,
        lasts: 0.60,
    });
    if face_up {
        path = path.departing(Departure {
            offset: Vec3::X * (21. * scale),
            share: 0.18,
        });
    }
    let entity = commands
        .spawn((
            ChildOf(board),
            place(from.min.x, from.min.y, from.width(), from.height()),
            ImageNode::new(if face_up {
                prints.back.clone()
            } else {
                prints.front(player.identity)
            }),
            GlobalZIndex(5),
            Pickable::IGNORE,
        ))
        .id();
    Flight {
        entity,
        destination: zone,
        player,
        path,
        elapsed: 0.,
    }
}

pub(super) fn draw(
    mut commands: Commands,
    session: Res<Session>,
    prints: Res<Prints>,
    layout: Query<(Entity, &Board, &Layout)>,
    mut playback: ResMut<Playback>,
    offers: Query<(&OfferFace, &mut Visibility), Without<PileTop>>,
    piles: Query<(&PileTop, &mut Visibility), Without<OfferFace>>,
) {
    let Some((board, shown, layout)) = layout.single().ok() else {
        *playback = Playback::default();
        return;
    };
    let revision = session.game.revision();
    // Keyboard commits can arrive after the UI draws for this frame.
    if shown.revision != revision {
        return;
    }
    if playback.board == Some(board)
        && playback
            .previous
            .as_ref()
            .is_some_and(|view| view.revision == revision)
    {
        return;
    }
    playback.flights.clear();
    playback.board = Some(board);
    let current = session.game.observe(session.seat);
    let previous = playback.previous.replace(current);
    let (Some(current), Some(previous)) = (&playback.previous, previous) else {
        return;
    };
    if previous.revision.checked_add(1) != Some(current.revision) {
        return;
    }

    let former = Layout::new(
        Vec2::new(layout.width, layout.height),
        previous.demotion_pending,
    );
    let mut flights = Vec::new();
    match current.events.last().copied() {
        Some(Event::Signed {
            seat,
            card,
            slot,
            replacement,
            ..
        }) => {
            let offer = &current.offers[usize::from(slot - 1)];
            debug_assert_eq!(offer.player.card, replacement);
            flights.push(draw_flight(
                &mut commands,
                board,
                card,
                former.face(slot),
                layout.club_pile(seat, 0),
                Zone::Lineup(seat),
                &prints,
                layout.scale,
            ));
            flights.push(draw_flight(
                &mut commands,
                board,
                replacement,
                former.deck_top(),
                layout.face(slot),
                Zone::Offer(replacement),
                &prints,
                layout.scale,
            ));
        }
        Some(Event::Demoted { seat, card }) => {
            let source = previous
                .rosters
                .iter()
                .find(|roster| roster.seat == seat)
                .and_then(|roster| roster.in_play.iter().position(|player| player.card == card));
            if let Some(index) = source {
                flights.push(draw_flight(
                    &mut commands,
                    board,
                    card,
                    former.in_play_face(index as u8),
                    layout.club_pile(seat, 3),
                    Zone::Minors(seat),
                    &prints,
                    layout.scale,
                ));
            }
        }
        _ => {}
    }
    set_destinations(offers, piles, &flights);
    playback.flights = flights;
}

fn set_destinations(
    mut offers: Query<(&OfferFace, &mut Visibility), Without<PileTop>>,
    mut piles: Query<(&PileTop, &mut Visibility), Without<OfferFace>>,
    flights: &[Flight],
) {
    for (offer, mut visibility) in &mut offers {
        let moving = flights
            .iter()
            .any(|flight| flight.destination == Zone::Offer(offer.0));
        visibility.set_if_neq(if moving {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    for (pile, mut visibility) in &mut piles {
        let moving =
            pile.count == 1 && flights.iter().any(|flight| flight.destination == pile.zone);
        visibility.set_if_neq(if moving {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
}

pub(super) fn animate(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut playback: ResMut<Playback>,
    prints: Res<Prints>,
    mut animations: Query<(&mut Node, &mut ImageNode)>,
    offers: Query<(&OfferFace, &mut Visibility), Without<PileTop>>,
    piles: Query<(&PileTop, &mut Visibility), Without<OfferFace>>,
) {
    if !playback.moving() {
        return;
    }
    playback.flights.retain_mut(|flight| {
        flight.elapsed += time.delta_secs();
        if keys.just_pressed(KeyCode::Space) || flight.path.finished(flight.elapsed) {
            commands.entity(flight.entity).despawn();
            return false;
        }
        if let Ok((mut node, mut image)) = animations.get_mut(flight.entity) {
            let pose = flight.path.pose(flight.elapsed);
            let wide = (pose.rotation * Vec3::X).x;
            let width = pose.translation.z * wide.abs().max(0.013);
            let height = pose.translation.z * CARD_ASPECT;
            *node = place(
                pose.translation.x - width / 2.,
                -pose.translation.y - height / 2.,
                width,
                height,
            );
            image.image = if wide >= 0. {
                prints.front(flight.player.identity)
            } else {
                prints.back.clone()
            };
        }
        true
    });
    if !playback.moving() {
        set_destinations(offers, piles, &playback.flights);
    }
}
