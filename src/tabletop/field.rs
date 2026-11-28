pub(super) mod hand;

use super::{
    Prints,
    controls::{
        Board, CREAM, DARK, FIELD_LABEL, GOLD, INK, Input, MUTED, PAPER, Selection, col, picture,
        row, short_button, text,
    },
};
use baseball_highlights_unfolded::{
    Action, Decision, Diamond, Event, FieldView, GlovePreview, Observation, Player, Seat,
};
use bevy::{picking::prelude::Pickable, prelude::*};
use unfolded_tabletop::seat::Name;

const BACKGROUND: Color = Color::srgb_u8(21, 40, 47);
const DIVIDER: Color = Color::srgb_u8(61, 84, 87);
const DESIGN: Vec2 = Vec2::new(1920., 1000.);

#[derive(Clone, Copy, Component)]
pub(super) struct Layout {
    size: Vec2,
    origin: Vec2,
    scale: f32,
}

impl Layout {
    pub(super) fn new(size: Vec2) -> Self {
        let ratio = size / DESIGN;
        let scale = ratio.min_element();
        Self {
            size,
            origin: (size - DESIGN * scale) / 2.,
            scale,
        }
    }

    pub(super) fn stadium(self) -> Rect {
        self.rect(36., 245., 1034., 603.)
    }

    fn rect(self, x: f32, y: f32, width: f32, height: f32) -> Rect {
        let top = self.origin + Vec2::new(x, y) * self.scale;
        Rect::from_corners(top, top + Vec2::new(width, height) * self.scale)
    }

    fn place(self, x: f32, y: f32, width: f32, height: f32) -> Node {
        position(self.rect(x, y, width, height))
    }
}

fn position(rect: Rect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(rect.min.x),
        top: px(rect.min.y),
        width: px(rect.width()),
        height: px(rect.height()),
        ..default()
    }
}

fn area(root: &mut ChildSpawnerCommands, min: Vec2, max: Vec2, color: Color) {
    root.spawn((
        position(Rect::from_corners(min, max)),
        BackgroundColor(color),
        Pickable::IGNORE,
    ));
}

pub(super) fn club_name(seat: Seat) -> &'static str {
    match seat {
        Seat::SanFrancisco => "San Francisco",
        Seat::NewYork => "New York",
    }
}
const BASES: [&str; 3] = ["First", "Second", "Third"];

fn index(seat: Seat) -> usize {
    match seat {
        Seat::SanFrancisco => 0,
        Seat::NewYork => 1,
    }
}

fn score(diamonds: &[Diamond; 2]) -> String {
    format!("NY {}  -  SF {}", diamonds[1].score, diamonds[0].score)
}

fn runners(diamond: &Diamond) -> Vec<String> {
    BASES
        .iter()
        .zip(diamond.bases)
        .filter_map(|(base, runner)| {
            let runner = runner?;
            let name = Player::describe(runner.card).map_or("Runner", |player| player.name);
            Some(format!("{base} / {name} ({})", runner.speed.label()))
        })
        .collect()
}

fn face(
    spawner: &mut ChildSpawnerCommands,
    prints: &Prints,
    player: Player,
    caption: &str,
    scale: f32,
) {
    spawner.spawn(col(5. * scale)).with_children(|card| {
        text(card, caption, 22. * scale, FIELD_LABEL);
        picture(
            card,
            prints.front(player.card.identity),
            202. * scale,
            281. * scale,
        );
        text(card, player.name, 25. * scale, CREAM);
    });
}

fn compare(
    panel: &mut ChildSpawnerCommands,
    view: &Observation,
    field: &FieldView,
    targets: &[GlovePreview],
    selection: Option<Selection>,
    scale: f32,
) {
    text(panel, "Glove target", 26. * scale, CREAM);
    panel.spawn(row(12. * scale)).with_children(|choices| {
        for preview in targets {
            let target = preview.target;
            let selected = selection == Some(Selection::Glove(target));
            choices
                .spawn((
                    Node {
                        flex_grow: 1.,
                        flex_basis: px(0.),
                        min_width: px(0.),
                        padding: UiRect::all(px(11. * scale)),
                        ..col(7. * scale)
                    },
                    BackgroundColor(Color::srgb(0.97, 0.91, 0.80)),
                ))
                .with_children(|choice| {
                    short_button(
                        choice,
                        Name::Component(format!("glove-hit-box-{}", target.number)),
                        format!(
                            "Preview box {} / {}",
                            target.number,
                            preview.cancelled.label()
                        ),
                        Input::Inspect(Selection::Glove(target)),
                        if selected { GOLD } else { PAPER },
                    );
                    let hits = preview
                        .surviving
                        .iter()
                        .map(|threat| threat.hit.label())
                        .collect::<Vec<_>>()
                        .join(" then ");
                    text(choice, format!("Survives   {hits}"), 23. * scale, INK);
                    text(
                        choice,
                        format!("Projected   {}", score(&preview.diamonds)),
                        23. * scale,
                        INK,
                    );
                    let positions = runners(&preview.diamonds[index(field.batting)]);
                    if positions.is_empty() {
                        text(choice, "Bases empty", 22. * scale, MUTED);
                    }
                    for position in positions {
                        text(choice, position, 22. * scale, MUTED);
                    }
                });
        }
    });
    if let Some(preview) = targets
        .iter()
        .find(|preview| selection == Some(Selection::Glove(preview.target)))
    {
        short_button(
            panel,
            Name::Choice("commit-glove-target".into()),
            format!("Commit Glove / Cancel box {}", preview.target.number),
            Input::Submit(view.submission(Action::CancelHit {
                target: preview.target,
            })),
            GOLD,
        );
    }
}

fn field_event(event: Event) -> Option<String> {
    match event {
        Event::GloveCancelled { seat, threat, .. } => {
            let player = Player::describe(threat.target.card)?;
            Some(format!(
                "{} used Glove to cancel {}'s {} (box {}).",
                seat.abbreviation(),
                player.name,
                threat.hit.label().to_lowercase(),
                threat.target.number,
            ))
        }
        Event::CardPlayed { seat, card } => {
            let player = Player::describe(card)?;
            let ability = if player.immediate_action.is_empty() {
                String::new()
            } else {
                format!(" with {}", player.immediate_action)
            };
            Some(format!(
                "{} played {}{ability}.",
                seat.abbreviation(),
                player.name
            ))
        }
        Event::FastballCancelled { seat } => Some(format!(
            "{} cancelled all hits with Fastball.",
            seat.abbreviation()
        )),
        Event::ImmediateHit {
            seat,
            card,
            hit,
            runs,
        } => Some(format!(
            "{} / {} for {} / {runs} runs.",
            Player::describe(card)?.name,
            hit.label(),
            seat.abbreviation(),
        )),
        Event::HitResolved { seat, threat, runs } => {
            let name = Player::describe(threat.target.card)?.name;
            let suffix = if runs == 1 { "run" } else { "runs" };
            Some(format!(
                "{name} / {} for {} / {runs} {suffix}.",
                threat.hit.label(),
                seat.abbreviation()
            ))
        }
        Event::ThreatsPlaced { seat, card, count } => {
            let player = Player::describe(card)?;
            let threats = if count == 0 {
                "has no threatened hits".to_owned()
            } else {
                format!(
                    "threatens {}",
                    player.threatened_hits.join(" then ").to_lowercase()
                )
            };
            Some(format!(
                "{}'s {} {threats}.",
                seat.abbreviation(),
                player.name
            ))
        }
        _ => None,
    }
}

fn game_header(
    root: &mut ChildSpawnerCommands,
    layout: Layout,
    view: &Observation,
    game: &FieldView,
    prints: &Prints,
) {
    let s = layout.scale;
    let title = match view.standing {
        Some(standing) => format!("Expert / Game {}", standing.game),
        None => "Glove practice".into(),
    };
    root.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::bottom(px(2. * s)),
            padding: UiRect::horizontal(px(34. * s)),
            ..layout.place(0., 0., DESIGN.x, 120.)
        },
        BackgroundColor(DARK),
        BorderColor::all(GOLD),
        Pickable::IGNORE,
    ))
    .with_children(|header| {
        text(header, title, 37. * s, CREAM);
        header.spawn(col(1. * s)).with_children(|scoreboard| {
            text(scoreboard, "RUNS", 19. * s, FIELD_LABEL);
            scoreboard
                .spawn(Node {
                    align_items: AlignItems::Center,
                    ..row(19. * s)
                })
                .with_children(|scores| {
                    for team in [Seat::SanFrancisco, Seat::NewYork] {
                        let runs = game.diamonds[index(team)].score;
                        let host = team == game.home;
                        text(
                            scores,
                            format!("{}  {runs}", team.abbreviation()),
                            38. * s,
                            CREAM,
                        );
                        scores
                            .spawn((
                                Node {
                                    align_items: AlignItems::Center,
                                    padding: UiRect::axes(px(10. * s), px(4. * s)),
                                    border_radius: BorderRadius::all(px(6. * s)),
                                    ..row(5. * s)
                                },
                                BackgroundColor(if host { CREAM } else { DARK }),
                            ))
                            .with_children(|role| {
                                if host {
                                    picture(role, prints.pennant.clone(), 34. * s, 34. * s);
                                }
                                text(
                                    role,
                                    if host { "Home" } else { "Visitor" },
                                    25. * s,
                                    if host { INK } else { FIELD_LABEL },
                                );
                            });
                    }
                });
        });
        if let Some(standing) = view.standing {
            header.spawn(col(2. * s)).with_children(|series| {
                text(series, "SERIES WINS", 19. * s, FIELD_LABEL);
                text(
                    series,
                    format!("SF  {}     NY  {}", standing.wins[0], standing.wins[1]),
                    29. * s,
                    CREAM,
                );
            });
        }
        header.spawn(col(2. * s)).with_children(|viewer| {
            text(viewer, "VIEWING", 19. * s, FIELD_LABEL);
            text(viewer, club_name(view.seat), 29. * s, CREAM);
        });
    });
}

pub(super) fn threat_summary(game: &FieldView) -> Option<String> {
    let first = game.threatened.first()?;
    let who = Player::describe(first.target.card)?.name;
    let hits = game
        .threatened
        .iter()
        .map(|threat| format!("a {}", threat.hit.label().to_lowercase()))
        .collect::<Vec<_>>()
        .join(" then ");
    Some(format!(
        "{}'s {who} threatens {hits}.",
        club_name(game.batting)
    ))
}

fn opening(view: &Observation) -> bool {
    view.standing.is_some() && view.rosters.iter().all(|roster| roster.in_play.is_empty())
}

fn ballpark(root: &mut ChildSpawnerCommands, layout: Layout, game: &FieldView) {
    let s = layout.scale;
    root.spawn((layout.place(36., 151., 1034., 70.), Pickable::IGNORE))
        .with_children(|title| {
            text(
                title,
                format!("{} ballpark", club_name(game.home)),
                42. * s,
                CREAM,
            )
        });
    root.spawn((
        Node {
            align_items: AlignItems::Start,
            ..layout.place(36., 870., 1034., 91.)
        },
        Pickable::IGNORE,
    ))
    .with_children(|live| {
        for team in [Seat::SanFrancisco, Seat::NewYork] {
            live.spawn(Node {
                flex_basis: px(0.),
                flex_grow: 1.,
                ..col(3. * s)
            })
            .with_children(|club| {
                let diamond = &game.diamonds[index(team)];
                let positions = runners(diamond);
                text(club, club_name(team), 26. * s, CREAM);
                if positions.is_empty() {
                    text(club, "Bases empty", 23. * s, FIELD_LABEL);
                }
                for position in positions {
                    text(club, position, 22. * s, FIELD_LABEL);
                }
            });
        }
    });
}

pub(super) fn prompt(view: &Observation, game: &FieldView) -> String {
    if game.glove_pending {
        if view.acting_manager == Some(view.seat) {
            "Which hit will you cancel?".into()
        } else {
            format!("{} is choosing a Glove target", club_name(game.acting))
        }
    } else if view.acting_manager.is_none() {
        if view.standing.is_some() {
            "Game two opening review complete".into()
        } else {
            "Glove resolved".into()
        }
    } else if opening(view) {
        if view.acting_manager == Some(view.seat) {
            "Play your first player".into()
        } else {
            format!("{} opens as Visitor", club_name(game.home.opponent()))
        }
    } else {
        let response = !game.threatened.is_empty();
        match (view.acting_manager, response) {
            (Some(manager), true) if manager != view.seat => {
                format!("{} responds to the threat", club_name(manager))
            }
            (Some(manager), false) if manager != view.seat => {
                format!("{} chooses the next player", club_name(manager))
            }
            (_, true) => "Choose a response to the threat".into(),
            (_, false) => "Choose the next player".into(),
        }
    }
}

fn adjacent(
    root: &mut ChildSpawnerCommands,
    layout: Layout,
    view: &Observation,
    game: &FieldView,
    prints: &Prints,
    selection: Option<Selection>,
    error: Option<&str>,
) {
    let s = layout.scale;
    root.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2. * s),
            ..layout.place(1120., 146., 768., 108.)
        },
        Pickable::IGNORE,
    ))
    .with_children(|heading| {
        if let Some(manager) = view.acting_manager {
            text(
                heading,
                format!("ACTING MANAGER  /  {}", club_name(manager)),
                20. * s,
                GOLD,
            );
        }
        text(heading, prompt(view, game), 41. * s, CREAM);
        if !game.glove_pending
            && let Some(threat) = threat_summary(game)
        {
            text(heading, threat, 22. * s, FIELD_LABEL);
        } else if opening(view) {
            text(
                heading,
                format!(
                    "{} visits and plays first.",
                    club_name(game.home.opponent())
                ),
                22. * s,
                FIELD_LABEL,
            );
        }
    });
    if let Some(Decision::PlayCard { hand }) = &view.decision {
        hand::draw(root, layout, view, hand, prints);
        if let Some(error) = error {
            root.spawn((layout.place(36., 970., 1034., 27.), Pickable::IGNORE))
                .with_children(|status| text(status, error, 19. * s, GOLD));
        }
        return;
    }
    root.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(11. * s),
            ..layout.place(1120., 258., 768., 718.)
        },
        Pickable::IGNORE,
    ))
    .with_children(|panel| {
        if let Some(play) = &game.play {
            let defender = if game.glove_pending {
                game.acting
            } else {
                game.batting
            };
            panel.spawn(row(31. * s)).with_children(|sources| {
                face(
                    sources,
                    prints,
                    play.batter,
                    &format!("{} play", defender.opponent().abbreviation()),
                    s,
                );
                let action = if view.standing.is_none() || game.glove_pending {
                    "Glove"
                } else {
                    "play"
                };
                face(
                    sources,
                    prints,
                    play.defender,
                    &format!("{} {action}", defender.abbreviation()),
                    s,
                );
            });
        }
        if let Some(Decision::Glove { targets }) = &view.decision {
            compare(panel, view, game, targets, selection, s);
        } else {
            let recent: Vec<_> = view
                .events
                .iter()
                .rev()
                .copied()
                .map_while(field_event)
                .take(4)
                .collect();
            for entry in recent.into_iter().rev() {
                text(panel, entry, 22. * s, FIELD_LABEL);
            }
        }
        if let Some(error) = error {
            text(panel, error, 21. * s, GOLD);
        }
    });
}

pub fn draw(
    commands: &mut Commands,
    layout: Layout,
    view: &Observation,
    field: &FieldView,
    prints: &Prints,
    selection: Option<Selection>,
    error: Option<&str>,
) {
    commands
        .spawn((
            Board {
                revision: view.revision,
            },
            layout,
            Node {
                width: percent(100.),
                height: percent(100.),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            // Leave this exact rectangle clear for the independent stadium camera.
            let park = layout.stadium();
            for (min, max) in [
                (Vec2::ZERO, Vec2::new(layout.size.x, park.min.y)),
                (Vec2::new(0., park.min.y), Vec2::new(park.min.x, park.max.y)),
                (
                    Vec2::new(park.max.x, park.min.y),
                    Vec2::new(layout.size.x, park.max.y),
                ),
                (Vec2::new(0., park.max.y), layout.size),
            ] {
                area(root, min, max, BACKGROUND);
            }
            let divider = layout.rect(1094., 146., 2., 832.);
            area(root, divider.min, divider.max, DIVIDER);
            game_header(root, layout, view, field, prints);
            ballpark(root, layout, field);
            adjacent(root, layout, view, field, prints, selection, error);
        });
}
