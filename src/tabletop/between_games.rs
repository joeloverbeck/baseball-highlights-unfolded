//! The completed Buy Round and its public Dugouts.
use super::{
    Prints,
    controls::{self, CREAM, FIELD_LABEL, GOLD, INK, Input, Selection},
    revenue::ReadingControl,
};
use baseball_highlights_unfolded::{
    Action, Decision, Event, NextGame, Observation, Player, Roster, Seat,
};
use bevy::{picking::prelude::Pickable, prelude::*};
use serde::Serialize;
use unfolded_tabletop::seat::Name as SeatName;

const BACKGROUND: Color = Color::srgb_u8(21, 40, 47);
const CLUB_PANEL: Color = Color::srgb_u8(36, 60, 64);

#[derive(Serialize)]
pub(super) struct Inspection<'a> {
    seat: Seat,
    players: &'a [Player],
    player: Option<Player>,
}

pub(super) fn inspection(
    view: &Observation,
    selection: Option<Selection>,
) -> Option<Inspection<'_>> {
    view.next_game?;
    let (seat, selected) = match selection? {
        Selection::Dugout(seat) => (seat, None),
        Selection::DugoutCard { seat, card } => (seat, Some(card)),
        _ => return None,
    };
    let roster = view.rosters.iter().find(|roster| roster.seat == seat)?;
    Some(Inspection {
        seat,
        players: &roster.dugout,
        player: roster
            .dugout
            .iter()
            .copied()
            .find(|player| Some(player.card) == selected),
    })
}

fn club(seat: Seat) -> &'static str {
    match seat {
        Seat::SanFrancisco => "San Francisco",
        Seat::NewYork => "New York",
    }
}

fn text(parent: &mut ChildSpawnerCommands, words: impl Into<String>, size: f32, color: Color) {
    parent.spawn((
        Text::new(words),
        TextFont {
            font_size: FontSize::Vh(size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    ));
}

fn face(
    parent: &mut ChildSpawnerCommands,
    player: Player,
    prints: &Prints,
    height: f32,
    width: f32,
) {
    parent.spawn((
        ImageNode::new(prints.front(player.card.identity)),
        Node {
            height: vh(height),
            max_width: vw(width),
            flex_shrink: 0.,
            ..default()
        },
        Pickable::IGNORE,
    ));
}

fn region(left: f32, top: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: percent(left),
        top: percent(top),
        width: percent(width),
        height: percent(height),
        ..default()
    }
}

fn button(
    parent: &mut ChildSpawnerCommands,
    name: SeatName,
    label: impl Into<String>,
    input: Input,
    node: Node,
    pennant: Option<&Handle<Image>>,
) {
    let label = label.into();
    let id = controls::button(
        parent,
        name,
        &label,
        input,
        if pennant.is_some() { GOLD } else { CREAM },
        Node {
            column_gap: vh(1.2),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..node
        },
    );
    parent.commands().entity(id).with_children(|button| {
        if let Some(pennant) = pennant {
            button.spawn((
                ImageNode::new(pennant.clone()),
                Node {
                    width: vh(5.5),
                    height: vh(5.5),
                    ..default()
                },
                Pickable::IGNORE,
            ));
        }
        text(button, label, 2.25, INK);
    });
}

fn roster(
    root: &mut ChildSpawnerCommands,
    roster: &Roster,
    view: &Observation,
    prints: &Prints,
    home: Seat,
    left: f32,
) {
    let seat = roster.seat;
    let earned = view
        .revenue
        .iter()
        .find(|revenue| revenue.seat == seat)
        .map_or(0, |r| r.total);
    let signings = view.events.iter().filter_map(|event| match *event {
        Event::Signed {
            seat: owner,
            card,
            paid,
            ..
        } if owner == seat => Some((card, paid)),
        _ => None,
    });
    let spent: u8 = signings.clone().map(|(_, paid)| paid).sum();
    let latest = signings.last().and_then(|(card, _)| Player::describe(card));
    let demotions: Vec<_> = view
        .events
        .iter()
        .filter_map(|event| match *event {
            Event::Demoted { seat: owner, card } if owner == seat => Player::describe(card),
            _ => None,
        })
        .collect();
    let dugout = roster.dugout.len();

    root.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: vh(0.75),
            border_radius: BorderRadius::all(vh(0.8)),
            padding: UiRect::axes(vw(1.2), vh(1.6)),
            ..region(left, 25., 31., 53.)
        },
        BackgroundColor(CLUB_PANEL),
        Pickable::IGNORE,
    ))
    .with_children(|panel| {
        text(panel, club(seat), 3.2, CREAM);
        text(
            panel,
            if seat == home { "Home" } else { "Visitor" },
            2.05,
            GOLD,
        );
        panel
            .spawn(Node {
                align_items: AlignItems::Center,
                column_gap: vw(1.25),
                height: vh(25.),
                ..default()
            })
            .with_children(|buying| {
                if let Some(player) = latest {
                    face(buying, player, prints, 23.5, 10.);
                }
                buying
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: vh(0.65),
                        min_width: px(0.),
                        ..default()
                    })
                    .with_children(|record| {
                        if let Some(player) = latest {
                            text(record, format!("Signed {}", player.name), 2.4, CREAM);
                        } else {
                            text(record, "No signings", 2.4, CREAM);
                        }
                        text(
                            record,
                            format!("${earned} earned, ${spent} spent"),
                            1.95,
                            CREAM,
                        );
                        if earned > spent {
                            text(
                                record,
                                format!("${} expired", earned - spent),
                                1.85,
                                FIELD_LABEL,
                            );
                        }
                        match demotions.as_slice() {
                            [player] => text(
                                record,
                                format!("{} to Minor Leagues", player.name),
                                1.75,
                                FIELD_LABEL,
                            ),
                            several if !several.is_empty() => {
                                text(
                                    record,
                                    format!("{} to Minor Leagues", several.len()),
                                    1.75,
                                    FIELD_LABEL,
                                );
                            }
                            _ => {}
                        }
                    });
            });
        text(
            panel,
            format!(
                "{} in Lineup + {dugout} in Dugout = {} active",
                roster.lineup, roster.active
            ),
            1.75,
            CREAM,
        );
        button(
            panel,
            SeatName::Component(format!(
                "open-dugout-{}",
                seat.abbreviation().to_ascii_lowercase()
            )),
            format!("Inspect {} Dugout ({dugout} face up)", seat.abbreviation()),
            Input::Inspect(Selection::Dugout(seat)),
            region(4., 84., 92., 11.),
            None,
        );
    });
}

fn matchup(root: &mut ChildSpawnerCommands, view: &Observation, next: NextGame, prints: &Prints) {
    let home = next.standing.home;
    for (team, left) in view.rosters.iter().zip([2., 67.]) {
        roster(root, team, view, prints, home, left);
    }
    root.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: vh(1.),
            ..region(34., 34., 32., 27.)
        },
        Pickable::IGNORE,
    ))
    .with_children(|center| {
        let [sf, ny] = next.standing.wins;
        text(center, format!("Game {}", next.standing.game), 7.2, CREAM);
        text(center, format!("{} hosts", club(home)), 3.45, CREAM);
        text(
            center,
            format!("Series wins  SF {sf} | NY {ny}"),
            2.3,
            FIELD_LABEL,
        );
        text(
            center,
            format!("{} plays first", club(next.first_player)),
            2.3,
            FIELD_LABEL,
        );
    });
    if view.decision == Some(Decision::StartGame) {
        button(
            root,
            SeatName::Choice("start-game-two".into()),
            "Start Game 2",
            Input::Submit(view.submission(Action::StartGame)),
            region(39., 64., 22., 8.),
            Some(&prints.pennant),
        );
    } else if let Some(manager) = view.acting_manager {
        root.spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..region(34., 64., 32., 8.)
            },
            Pickable::IGNORE,
        ))
        .with_children(|wait| {
            text(
                wait,
                format!("Waiting for {} to start", club(manager)),
                2.25,
                GOLD,
            );
        });
    }
}

fn dugout(root: &mut ChildSpawnerCommands, reading: Inspection<'_>, prints: &Prints) {
    root.spawn((
        Node {
            justify_content: JustifyContent::Center,
            ..region(2., 15., 96., 8.)
        },
        Pickable::IGNORE,
    ))
    .with_children(|title| {
        text(
            title,
            format!(
                "{} Dugout ({} face up)",
                club(reading.seat),
                reading.players.len()
            ),
            3.5,
            CREAM,
        );
    });
    if let Some(player) = reading.player {
        root.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: vh(1.1),
                ..region(4., 23., 92., 58.)
            },
            Pickable::IGNORE,
        ))
        .with_children(|detail| {
            face(detail, player, prints, 49., 45.);
            text(detail, player.name, 2.75, CREAM);
        });
        button(
            root,
            SeatName::Component("close-inspection".into()),
            "Back to Dugout",
            Input::Inspect(Selection::Dugout(reading.seat)),
            region(38., 84., 24., 6.),
            None,
        );
        return;
    }
    root.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            ..region(4., 26., 92., 52.)
        },
        Pickable::IGNORE,
    ))
    .with_children(|faces| {
        let width = 88. / reading.players.len().max(5) as f32;
        for (index, &player) in reading.players.iter().enumerate() {
            let card = player.card;
            let id = controls::button(
                faces,
                SeatName::Component(format!("inspect-{}", card.identity)),
                format!("Inspect {}, position {}", player.name, index + 1),
                Input::Inspect(Selection::DugoutCard {
                    seat: reading.seat,
                    card,
                }),
                CLUB_PANEL,
                Node {
                    width: vw(width),
                    min_width: px(0.),
                    padding: UiRect::all(vh(0.75)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: vh(0.85),
                    ..default()
                },
            );
            faces
                .commands()
                .entity(id)
                .insert(ReadingControl)
                .with_children(|tile| {
                    face(tile, player, prints, 43., width - 1.5);
                    text(tile, player.name, 1.95, CREAM);
                });
        }
    });
    button(
        root,
        SeatName::Component("close-dugout".into()),
        "Close Dugout",
        Input::Close,
        region(38., 84., 24., 6.),
        None,
    );
}

pub(super) fn draw(
    root: &mut ChildSpawnerCommands,
    view: &Observation,
    next: NextGame,
    prints: &Prints,
    selection: Option<Selection>,
    error: Option<&str>,
) {
    root.spawn((
        Node {
            width: percent(100.),
            height: percent(100.),
            ..default()
        },
        BackgroundColor(BACKGROUND),
        Pickable::IGNORE,
    ))
    .with_children(|screen| {
        screen.spawn((
            region(0., 0., 100., 0.7),
            BackgroundColor(GOLD),
            Pickable::IGNORE,
        ));
        screen
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: vh(0.8),
                    ..region(2., 3., 60., 11.)
                },
                Pickable::IGNORE,
            ))
            .with_children(|heading| {
                text(
                    heading,
                    format!("Expert Series | {} manager", view.seat.abbreviation()),
                    1.9,
                    FIELD_LABEL,
                );
                text(heading, "Buy Round complete", 4.5, CREAM);
            });
        if let Some(reading) = inspection(view, selection) {
            dugout(screen, reading, prints);
        } else if selection == Some(Selection::History) {
            screen
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: vh(1.3),
                        ..region(4., 20., 92., 60.)
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|history| {
                    for &event in &view.events {
                        text(history, controls::event_text(event), 2.4, CREAM);
                    }
                });
            button(
                screen,
                SeatName::Component("close-history".into()),
                "Close history",
                Input::Close,
                region(38., 84., 24., 6.),
                None,
            );
        } else {
            matchup(screen, view, next, prints);
            screen.spawn((region(2., 84., 96., 5.), Pickable::IGNORE))
                .with_children(|note| text(
                    note,
                    "The remaining In Play cards move to each club's face-up Dugout at the end of buying.",
                    1.95,
                    FIELD_LABEL,
                ));
            button(
                screen,
                SeatName::Component("round-history".into()),
                "Round history",
                Input::Inspect(Selection::History),
                region(83., 4., 15., 5.),
                None,
            );
        }
        if let Some(error) = error {
            screen
                .spawn((region(2., 93., 96., 5.), Pickable::IGNORE))
                .with_children(|notice| text(notice, error, 2.05, GOLD));
        }
    });
}
