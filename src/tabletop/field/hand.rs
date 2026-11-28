//! The acting manager's hand. Inspection and Play use the same physical card.
use super::{BACKGROUND, Layout, position};
use crate::tabletop::{
    Prints, Session,
    card_focus::{self, Interaction, Part, Region, Source, Target},
    card_help,
    controls::{self, CREAM, GOLD, INK, Input, picture, text},
};
use baseball_highlights_unfolded::{Action, Decision, Observation, Player};
use bevy::{picking::prelude::Pickable, prelude::*};
use serde_json::{Value, json};
use unfolded_tabletop::seat::Name;

const WIDTH: f32 = 222.;
const HEIGHT: f32 = 309.;
const OUTLINE: f32 = 6.;
const CLEARANCE: f32 = 14.;
const PLAY_HEIGHT: f32 = 32.;
const FRAME: Color = Color::srgb_u8(81, 107, 101);
const FOCUS: Color = Color::srgb_u8(255, 242, 205);

fn face_rect(layout: Layout, index: u8) -> Rect {
    let x = 1130. + f32::from(index % 3) * 252.;
    let y = 258. + f32::from(index / 3) * 375.;
    layout.rect(x, y, WIDTH, HEIGHT)
}

fn player(target: Target, view: &Observation) -> Option<(u8, Player)> {
    let (Source::Hand(index, card), Some(Decision::PlayCard { hand })) =
        (target.source, &view.decision)
    else {
        return None;
    };
    let player = *hand.get(usize::from(index))?;
    (target.matches(view) && view.acting_manager == Some(view.seat) && player.card == card)
        .then_some((index, player))
}

pub(in crate::tabletop) fn reading(view: &Observation, target: Option<Target>) -> Option<Value> {
    let (index, player) = player(target?, view)?;
    Some(json!({
        "position": index + 1,
        "card": player.card,
        "name": player.name,
        "symbols": card_help::symbols(player).collect::<Vec<_>>(),
    }))
}

pub(super) fn draw(
    root: &mut ChildSpawnerCommands,
    layout: Layout,
    view: &Observation,
    hand: &[Player],
    prints: &Prints,
) {
    let s = layout.scale;
    for (index, &player) in hand.iter().enumerate() {
        let index = index as u8;
        let face = face_rect(layout, index);
        let region = Region {
            target: Target::new(view, Source::Hand(index, player.card)),
            part: Part::Face,
        };
        let button = controls::button(
            root,
            Name::Component(format!("inspect-{}", player.card.identity)),
            format!("Inspect {}, position {}", player.name, index + 1),
            Input::ReadCard(region),
            BACKGROUND,
            Node {
                border: UiRect::all(px(2. * s)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..position(face)
            },
        );
        root.commands()
            .entity(button)
            .insert((
                region,
                BorderColor::all(FRAME),
                Outline::new(px(4. * s), px(2. * s), Color::NONE),
            ))
            .with_children(|face| {
                picture(face, prints.front(player.card.identity), 218. * s, 305. * s);
            });
    }
}

#[derive(Component)]
pub(in crate::tabletop) struct Disclosure;

#[derive(Component)]
pub(in crate::tabletop) struct PlayButton;

fn disclose(
    root: &mut ChildSpawnerCommands,
    layout: Layout,
    view: &Observation,
    target: Target,
    index: u8,
    player: Player,
    prints: &Prints,
) {
    let s = layout.scale;
    let face = face_rect(layout, index);
    let gap = (OUTLINE + CLEARANCE) * s;
    let play_top = face.max.y + gap;
    let help_width = layout.rect(653., 0., 464., 0.);
    let help_height = if player.pinch_hitter { 285. } else { 158. } * s;
    let help = Rect::from_corners(
        Vec2::new(help_width.min.x, play_top - help_height),
        Vec2::new(help_width.max.x, play_top),
    );
    // Bevy excludes the exact edge of a hit rectangle. Overlap adjacent
    // hit regions by a logical pixel while preserving the painted clearance.
    let overlap = 1.;
    for rect in [
        Rect::from_corners(
            face.min.with_y(face.max.y - overlap),
            face.max.with_y(play_top + overlap),
        ),
        Rect::from_corners(
            Vec2::new(help.max.x - overlap, face.max.y + OUTLINE * s),
            Vec2::new(face.min.x + overlap, play_top),
        ),
    ] {
        card_focus::bridge(root, target, position(rect));
    }
    let region = Region {
        target,
        part: Part::Help,
    };
    let read = controls::button(
        root,
        Name::Component(format!("hand-help-{}", player.card.identity)),
        format!("{}'s card symbols", player.name),
        Input::ReadCard(region),
        CREAM,
        Node {
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(7. * s),
            padding: UiRect::all(px(14. * s)),
            ..position(help)
        },
    );
    root.commands()
        .entity(read)
        .insert((region, Outline::new(px(3. * s), px(0.), Color::NONE)))
        .with_children(|help| card_help::draw(help, prints, player, 1.4 * s));

    let play = controls::button(
        root,
        Name::Choice(format!("play-{}", player.card.identity)),
        format!("Play {}", player.name),
        Input::Submit(view.submission(Action::PlayCard { card: player.card })),
        GOLD,
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(3. * s)),
            ..position(Rect::from_corners(
                face.min.with_y(play_top),
                face.max.with_y(play_top + PLAY_HEIGHT * s),
            ))
        },
    );
    root.commands()
        .entity(play)
        .insert((
            Region {
                target,
                part: Part::Action,
            },
            PlayButton,
            BorderColor::all(GOLD),
        ))
        .with_children(|play| text(play, "Play", 23. * s, INK));
}

pub(in crate::tabletop) fn present(
    mut commands: Commands,
    layout: Query<(Entity, &Layout)>,
    interaction: Res<Interaction>,
    session: Res<Session>,
    prints: Res<Prints>,
    mut existing: Query<(&Region, &mut Outline)>,
    mut play_buttons: Query<(&Region, &mut BorderColor), With<PlayButton>>,
    disclosures: Query<Entity, With<Disclosure>>,
    mut shown: Local<Option<(Entity, Target)>>,
) {
    let visible = interaction
        .visible()
        .filter(|target| matches!(target.source, Source::Hand(..)));
    let next = layout
        .single()
        .ok()
        .zip(visible)
        .map(|((entity, _), target)| (entity, target));
    if *shown != next {
        *shown = next;
        for entity in &disclosures {
            commands.entity(entity).despawn();
        }
        if let Some((root, target)) = next
            && let Ok((_, layout)) = layout.get(root)
        {
            let view = session.game.observe(session.seat);
            if let Some((index, player)) = player(target, &view) {
                commands.entity(root).with_children(|root| {
                    root.spawn((
                        Disclosure,
                        Node {
                            position_type: PositionType::Absolute,
                            width: percent(100.),
                            height: percent(100.),
                            ..default()
                        },
                        ZIndex(2),
                        Pickable::IGNORE,
                    ))
                    .with_children(|layer| {
                        disclose(layer, *layout, &view, target, index, player, &prints)
                    });
                });
            }
        }
    }
    for (region, outline) in &mut existing {
        if matches!(region.target.source, Source::Hand(..)) {
            let color = if interaction.keyboard_focus() == Some(*region) {
                FOCUS
            } else if region.part == Part::Face && Some(region.target) == visible {
                GOLD
            } else {
                Color::NONE
            };
            outline
                .map_unchanged(|outline| &mut outline.color)
                .set_if_neq(color);
        }
    }
    for (region, mut border) in &mut play_buttons {
        let color = if interaction.keyboard_focus() == Some(*region) {
            FOCUS
        } else {
            GOLD
        };
        border.set_if_neq(BorderColor::all(color));
    }
}
