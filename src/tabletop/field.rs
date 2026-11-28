use super::{
    HEADER, Prints,
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

pub const HEIGHT: f32 = 304.;
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

fn face(spawner: &mut ChildSpawnerCommands, prints: &Prints, player: Player, caption: &str) {
    spawner.spawn(col(5.)).with_children(|card| {
        text(card, caption, 15., MUTED);
        picture(card, prints.faces[&player.printed_face].clone(), 148., 205.);
        text(card, player.name, 17., INK);
        text(card, player.card.identity.to_string(), 13., MUTED);
    });
}

fn compare(
    panel: &mut ChildSpawnerCommands,
    view: &Observation,
    field: &FieldView,
    targets: &[GlovePreview],
    selection: Option<Selection>,
) {
    text(panel, "Glove target", 25., INK);
    panel.spawn(row(10.)).with_children(|choices| {
        for preview in targets {
            let target = preview.target;
            let selected = selection == Some(Selection::Glove(target));
            choices
                .spawn((
                    Node {
                        flex_grow: 1.,
                        flex_basis: px(0.),
                        min_width: px(0.),
                        padding: UiRect::all(px(10.)),
                        ..col(7.)
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
                    text(choice, format!("Survives   {hits}"), 17., INK);
                    text(
                        choice,
                        format!("Projected   {}", score(&preview.diamonds)),
                        17.,
                        INK,
                    );
                    let positions = runners(&preview.diamonds[index(field.batting)]);
                    if positions.is_empty() {
                        text(choice, "Bases empty", 16., MUTED);
                    }
                    for position in positions {
                        text(choice, position, 16., MUTED);
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
        Event::GloveCancelled { seat, threat, .. } => Some(format!(
            "{} cancelled box {} / {}.",
            seat.abbreviation(),
            threat.target.number,
            threat.hit.label()
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
                "no hits".to_string()
            } else {
                player.threatened_hits.join(" then ")
            };
            Some(format!("{} threatens {threats}.", seat.abbreviation()))
        }
        _ => None,
    }
}

pub fn draw(
    commands: &mut Commands,
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
            Node {
                width: percent(100.),
                height: percent(100.),
                ..col(0.)
            },
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: percent(100.),
                    height: px(HEADER),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::horizontal(px(20.)),
                    ..default()
                },
                BackgroundColor(DARK),
                Pickable::IGNORE,
            ))
            .with_children(|header| {
                text(
                    header,
                    format!("Expert   /   {} home", field.home.abbreviation()),
                    21.,
                    CREAM,
                );
                text(header, score(&field.diamonds), 28., GOLD);
                text(
                    header,
                    format!("{} manager", view.seat.abbreviation()),
                    21.,
                    CREAM,
                );
            });
            root.spawn((
                Node {
                    height: px(HEIGHT),
                    flex_shrink: 0.,
                    padding: UiRect::axes(px(20.), px(16.)),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|live| {
                for team in [Seat::NewYork, Seat::SanFrancisco] {
                    live.spawn(Node {
                        width: px(195.),
                        ..col(6.)
                    })
                    .with_children(|club| {
                        let caption = if field.batting == team {
                            " / Batting"
                        } else {
                            ""
                        };
                        text(club, format!("{}{caption}", team.abbreviation()), 21., GOLD);
                        let positions = runners(&field.diamonds[index(team)]);
                        if positions.is_empty() {
                            text(club, "Bases empty", 16., FIELD_LABEL);
                        }
                        for position in positions {
                            text(club, position, 16., CREAM);
                        }
                        if team == field.batting && !field.threatened.is_empty() {
                            text(club, "Threatened hits", 16., FIELD_LABEL);
                            for threat in &field.threatened {
                                text(
                                    club,
                                    format!(
                                        "Box {} / {}",
                                        threat.target.number,
                                        threat.hit.label()
                                    ),
                                    16.,
                                    CREAM,
                                );
                            }
                        }
                    });
                }
            });
            root.spawn((
                Node {
                    flex_grow: 1.,
                    padding: UiRect::axes(px(22.), px(13.)),
                    ..row(22.)
                },
                BackgroundColor(PAPER),
            ))
            .with_children(|book| {
                book.spawn(row(14.)).with_children(|sources| {
                    face(sources, prints, field.play.batter, "NY play");
                    face(sources, prints, field.play.defender, "SF Glove");
                });
                book.spawn(Node {
                    flex_grow: 1.,
                    min_width: px(0.),
                    ..col(10.)
                })
                .with_children(|panel| {
                    if let Some(Decision::Glove { targets }) = &view.decision {
                        compare(panel, view, field, targets, selection);
                    } else if field.glove_pending {
                        text(
                            panel,
                            format!("{} is choosing a Glove target", field.acting.abbreviation()),
                            25.,
                            INK,
                        );
                    } else {
                        text(panel, "Glove resolved", 25., INK);
                        let recent: Vec<_> = view
                            .events
                            .iter()
                            .rev()
                            .copied()
                            .map_while(field_event)
                            .take(4)
                            .collect();
                        for entry in recent.into_iter().rev() {
                            text(panel, entry, 17., MUTED);
                        }
                    }
                    if let Some(error) = error {
                        text(panel, error, 17., INK);
                    }
                });
            });
        });
}
