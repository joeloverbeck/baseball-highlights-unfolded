//! Public In Play faces and the Glove decision beside the live stadium.
use super::{BACKGROUND, BASES, Layout, index, runners};
use crate::tabletop::{
    Prints, Session,
    card_focus::{self, Interaction, Part, Region, Source, Target},
    card_help,
    controls::{self, CREAM, DARK, FIELD_LABEL, GOLD, INK, Input, PAPER, Selection, picture, text},
};
use baseball_highlights_unfolded::{
    Action, FieldPlay, FieldView, GlovePreview, Observation, Player, RunnerAdvance, Seat,
};
use bevy::{picking::prelude::Pickable, prelude::*};
use serde_json::{Value, json};
use unfolded_tabletop::seat::Name;

const CARD_LEFT: [f32; 2] = [1150., 1496.];
const CARD_TOP: f32 = 285.;
const CARD_WIDTH: f32 = 202.;
const CARD_HEIGHT: f32 = 281.;
const FRAME: Color = Color::srgb_u8(81, 107, 101);
const FOCUS: Color = Color::srgb_u8(255, 242, 205);
const FACE_KEYS: [&str; 2] = ["inspect-glove-batter", "inspect-glove-defender"];
const HELP_KEYS: [&str; 2] = ["glove-batter-help", "glove-defender-help"];

fn matchup(game: &FieldView, play: &FieldPlay) -> [(Seat, Player); 2] {
    let defender = if game.glove_pending {
        game.acting
    } else {
        game.batting
    };
    [
        (defender.opponent(), play.batter),
        (defender, play.defender),
    ]
}

fn player(view: &Observation, target: Target) -> Option<(usize, Seat, Player)> {
    let Source::GloveFace(slot, card) = target.source else {
        return None;
    };
    let game = view.field.as_ref()?;
    let (seat, player) = *matchup(game, game.play.as_ref()?).get(usize::from(slot))?;
    (target.matches(view) && player.card == card).then_some((usize::from(slot), seat, player))
}

pub(in crate::tabletop) fn reading(view: &Observation, target: Option<Target>) -> Option<Value> {
    let (_, team, player) = player(view, target?)?;
    Some(json!({
        "team": team,
        "card": player.card,
        "name": player.name,
        "symbols": card_help::symbols(player).collect::<Vec<_>>(),
    }))
}

pub(super) fn draw_faces(
    root: &mut ChildSpawnerCommands,
    layout: Layout,
    view: &Observation,
    game: &FieldView,
    play: &FieldPlay,
    prints: &Prints,
) {
    let s = layout.scale;
    for (slot, (team, player)) in matchup(game, play).into_iter().enumerate() {
        let left = CARD_LEFT[slot];
        let role = if game
            .threatened
            .iter()
            .any(|hit| hit.target.card == player.card)
        {
            "Threatened hits"
        } else if slot == 0 {
            "Previous batter"
        } else if game.glove_pending {
            "Glove"
        } else {
            "Last play"
        };
        let accent = if team == Seat::SanFrancisco {
            GOLD
        } else {
            FIELD_LABEL
        };
        root.spawn((layout.place(left, 252., 320., 28.), Pickable::IGNORE))
            .with_children(|caption| {
                text(
                    caption,
                    format!("{} / {role}", team.abbreviation()),
                    23. * s,
                    accent,
                );
            });
        let region = Region {
            target: Target::new(view, Source::GloveFace(slot as u8, player.card)),
            part: Part::Face,
        };
        let face = controls::button(
            root,
            Name::Component(FACE_KEYS[slot].into()),
            format!(
                "Inspect {}, {} In Play / {role}",
                player.name,
                team.abbreviation()
            ),
            Input::ReadCard(region),
            BACKGROUND,
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(2. * s)),
                ..layout.place(left, CARD_TOP, CARD_WIDTH, CARD_HEIGHT)
            },
        );
        root.commands()
            .entity(face)
            .insert((
                region,
                BorderColor::all(FRAME),
                Outline::new(px(4. * s), px(2. * s), Color::NONE),
            ))
            .with_children(|face| {
                picture(
                    face,
                    prints.front(player.card.identity),
                    (CARD_WIDTH - 4.) * s,
                    (CARD_HEIGHT - 4.) * s,
                );
            });
        root.spawn((layout.place(left, 577., 322., 30.), Pickable::IGNORE))
            .with_children(|name| text(name, player.name, 26. * s, CREAM));
    }
}

fn movement(advance: &RunnerAdvance) -> String {
    let name = Player::describe(advance.runner.card).map_or("Runner", |player| player.name);
    let base = |index: usize| BASES[index - 1].to_lowercase();
    match (advance.from_base, advance.to_base) {
        (Some(from), None) => format!("{name} scores from {}.", base(from)),
        (None, None) => format!("{name} scores."),
        (None, Some(to)) => format!("{name} reaches {}.", base(to)),
        (Some(from), Some(to)) if from == to => format!("{name} holds {}.", base(from)),
        (Some(from), Some(to)) => format!("{name} advances from {} to {}.", base(from), base(to)),
    }
}

pub(super) fn draw_outcomes(
    root: &mut ChildSpawnerCommands,
    layout: Layout,
    view: &Observation,
    game: &FieldView,
    targets: &[GlovePreview],
    selection: Option<Selection>,
) {
    let s = layout.scale;
    let gap = 12.;
    let width = (772. + gap) / targets.len() as f32 - gap;
    for (slot, preview) in targets.iter().enumerate() {
        let hit_box = preview.target;
        let target = Target::new(view, Source::GloveBox(hit_box));
        let chosen = selection == Some(Selection::Glove(target));
        let left = 1120. + slot as f32 * (width + gap);
        let hit = preview.cancelled.label().to_lowercase();
        let batter = Player::describe(hit_box.card).map_or("Batter", |player| player.name);
        let caption = format!("Cancel {hit}");
        let surviving = preview
            .surviving
            .iter()
            .map(|threat| threat.hit.label().to_lowercase())
            .collect::<Vec<_>>()
            .join(" then ");
        let remains = format!("Survives: {surviving}");
        let score = format!(
            "Projected  NY {}  /  SF {}",
            preview.diamonds[1].score, preview.diamonds[0].score
        );
        let movements: Vec<_> = preview.runner_advances.iter().map(movement).collect();
        let mut bases = runners(&preview.diamonds[index(game.batting)]);
        if bases.is_empty() {
            bases.push("Bases empty".into());
        }
        let label = format!(
            "Preview canceling {batter}'s {hit}, box {}. {remains}. {} {score}. {}.",
            hit_box.number,
            movements.join(" "),
            bases.join("; "),
        );
        let outcome = controls::button(
            root,
            Name::Component(format!("glove-hit-box-{}", hit_box.number)),
            label,
            Input::Inspect(Selection::Glove(target)),
            PAPER,
            Node {
                padding: UiRect::axes(px(15. * s), px(12. * s)),
                border: UiRect::all(px(3. * s)),
                flex_direction: FlexDirection::Column,
                row_gap: px(5. * s),
                ..layout.place(left, 619., width, 247.)
            },
        );
        root.commands()
            .entity(outcome)
            .insert((
                Region {
                    target,
                    part: Part::Action,
                },
                BorderColor::all(if chosen { GOLD } else { PAPER }),
                Outline::new(px(4. * s), px(2. * s), Color::NONE),
            ))
            .with_children(|choice| {
                text(choice, caption, 27. * s, INK);
                text(choice, remains, 22. * s, INK);
                for change in movements {
                    text(choice, change, 20. * s, INK);
                }
                text(choice, score, 23. * s, INK);
                for base in bases {
                    text(choice, base, 19. * s, INK);
                }
            });
        if chosen {
            let action = controls::button(
                root,
                Name::Choice("commit-glove-target".into()),
                format!(
                    "Use Glove to cancel {batter}'s {hit}, box {}",
                    hit_box.number
                ),
                Input::Submit(view.submission(Action::CancelHit { target: hit_box })),
                GOLD,
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..layout.place(left, 880., width, 47.)
                },
            );
            root.commands()
                .entity(action)
                .insert((
                    Region {
                        target: Target::new(view, Source::UseGlove),
                        part: Part::Action,
                    },
                    Outline::new(px(4. * s), px(2. * s), Color::NONE),
                ))
                .with_children(|action| {
                    text(action, format!("Use Glove / Cancel {hit}"), 23. * s, DARK)
                });
        }
    }
}

#[derive(Component)]
pub(in crate::tabletop) struct Disclosure;

fn disclose(
    root: &mut ChildSpawnerCommands,
    layout: Layout,
    target: Target,
    slot: usize,
    player: Player,
    prints: &Prints,
) {
    let s = layout.scale;
    let left = CARD_LEFT[slot];
    let bottom = CARD_TOP + CARD_HEIGHT;
    let foot = 606.;
    let help_left = 666.;
    let help_width = 464.;
    let help_height = if player.pinch_hitter { 250. } else { 152. };
    for bridge in [
        layout.place(left, bottom - 2., CARD_WIDTH, foot - bottom + 2.),
        layout.place(
            help_left + help_width - 2.,
            bottom + 4.,
            left - (help_left + help_width) + 5.,
            foot - bottom - 4.,
        ),
    ] {
        card_focus::bridge(root, target, bridge);
    }
    let region = Region {
        target,
        part: Part::Help,
    };
    let help = controls::button(
        root,
        Name::Component(HELP_KEYS[slot].into()),
        format!("{}'s card symbols", player.name),
        Input::ReadCard(region),
        CREAM,
        Node {
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(7. * s),
            padding: UiRect::all(px(14. * s)),
            ..layout.place(help_left, foot - help_height, help_width, help_height)
        },
    );
    root.commands()
        .entity(help)
        .insert((region, Outline::new(px(3. * s), px(2. * s), Color::NONE)))
        .with_children(|help| card_help::draw(help, prints, player, 1.4 * s));
}

pub(in crate::tabletop) fn present(
    mut commands: Commands,
    layout: Query<(Entity, &Layout)>,
    interaction: Res<Interaction>,
    session: Res<Session>,
    prints: Res<Prints>,
    mut controls: Query<(&Region, &mut Outline)>,
    disclosures: Query<Entity, With<Disclosure>>,
    mut shown: Local<Option<(Entity, Target)>>,
) {
    let visible = interaction
        .visible()
        .filter(|target| matches!(target.source, Source::GloveFace(..)));
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
            && let Some((slot, _, player)) = player(&session.game.observe(session.seat), target)
        {
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
                .with_children(|layer| disclose(layer, *layout, target, slot, player, &prints));
            });
        }
    }
    for (region, outline) in &mut controls {
        if matches!(
            region.target.source,
            Source::GloveFace(..) | Source::GloveBox(_) | Source::UseGlove
        ) {
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
}
