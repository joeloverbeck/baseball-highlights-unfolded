//! Purchase-order view of each team's public In Play earnings.
use super::{
    Prints,
    controls::{self, CREAM, DARK, FIELD_LABEL, GOLD, INK, Input, PAPER, Selection},
};
use baseball_highlights_unfolded::{Action, Decision, Observation, Player, Revenue, Seat};
use bevy::{picking::prelude::Pickable, prelude::*};
use unfolded_tabletop::seat::Name as SeatName;

const LEDGER: Color = Color::srgb(0.14, 0.23, 0.25);
const RULE: Color = Color::srgb(0.28, 0.39, 0.39);
pub(super) const READ_HIGHLIGHT: Color = Color::srgb(0.25, 0.37, 0.39);

#[derive(Component)]
pub(super) struct ReadingControl;

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

fn face(parent: &mut ChildSpawnerCommands, image: Handle<Image>, height: f32, max_width: f32) {
    parent.spawn((
        ImageNode::new(image),
        Node {
            height: vh(height),
            max_width: vw(max_width),
            ..default()
        },
        Pickable::IGNORE,
    ));
}

fn horizontal(gap: f32) -> Node {
    Node {
        column_gap: vw(gap),
        align_items: AlignItems::Center,
        ..default()
    }
}

fn vertical(gap: f32) -> Node {
    Node {
        row_gap: vh(gap),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

fn reveal_button(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    label: String,
    action: Input,
    color: Color,
) {
    let id = controls::button(
        parent,
        SeatName::Component(name.into()),
        &label,
        action,
        color,
        Node {
            padding: UiRect::axes(vw(0.8), vh(0.6)),
            ..default()
        },
    );
    parent
        .commands()
        .entity(id)
        .with_children(|button| text(button, label, 1.8, INK));
}

pub(super) fn inspected_contributor(
    view: &Observation,
    selection: Option<Selection>,
) -> Option<Player> {
    let Some(Selection::PublicCard(card)) = selection.filter(|_| view.first_buyer.is_none()) else {
        return None;
    };
    view.rosters
        .iter()
        .flat_map(|roster| &roster.in_play)
        .find(|player| player.card == card)
        .copied()
}

fn market(root: &mut ChildSpawnerCommands, view: &Observation, prints: &Prints) {
    root.spawn(Node {
        height: vh(35.),
        justify_content: JustifyContent::SpaceBetween,
        ..horizontal(0.8)
    })
    .with_children(|market| {
        market
            .spawn(Node {
                align_items: AlignItems::Start,
                ..vertical(0.6)
            })
            .with_children(|deck| {
                face(deck, prints.back.clone(), 18., 7.);
                text(deck, "Free Agent Deck", 1.7, CREAM);
                text(
                    deck,
                    format!("{} remaining", view.free_agent_deck),
                    1.6,
                    FIELD_LABEL,
                );
            });
        for offer in &view.offers {
            let player = offer.player;
            let id = controls::button(
                market,
                SeatName::Component(format!("inspect-slot-{}", offer.slot)),
                format!("Inspect {}", player.name),
                Input::Inspect(Selection::Offer {
                    slot: offer.slot,
                    card: player.card,
                }),
                DARK,
                Node {
                    align_items: AlignItems::Start,
                    padding: UiRect::all(vh(0.4)),
                    ..vertical(0.6)
                },
            );
            market
                .commands()
                .entity(id)
                .insert(ReadingControl)
                .with_children(|offer_card| {
                    face(offer_card, prints.front(player.card.identity), 27., 12.6);
                    text(offer_card, format!("${}", offer.cost), 2.0, CREAM);
                });
        }
    });
}

fn ledger(ledgers: &mut ChildSpawnerCommands, revenue: &Revenue, selection: Option<Selection>) {
    ledgers
        .spawn((
            Node {
                flex_basis: percent(0.),
                flex_grow: 1.,
                align_self: AlignSelf::Stretch,
                padding: UiRect::axes(vw(1.05), vh(1.1)),
                border_radius: BorderRadius::all(vh(0.75)),
                ..vertical(0.85)
            },
            BackgroundColor(LEDGER),
        ))
        .with_children(|ledger| {
            ledger
                .spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..horizontal(0.7)
                })
                .with_children(|heading| {
                    text(heading, club(revenue.seat), 2.75, CREAM);
                    text(heading, format!("${} earned", revenue.total), 2.45, GOLD);
                });
            ledger
                .spawn(Node {
                    align_items: AlignItems::Start,
                    ..horizontal(1.0)
                })
                .with_children(|entries| {
                    let column_length = revenue.contributions.len().div_ceil(2).max(1);
                    for contributions in revenue.contributions.chunks(column_length) {
                        entries
                            .spawn(Node {
                                flex_basis: percent(0.),
                                flex_grow: 1.,
                                ..vertical(0.4)
                            })
                            .with_children(|column| {
                                for entry in contributions {
                                    let player = Player::describe(entry.card)
                                        .expect("revenue entries are public");
                                    let target = Selection::PublicCard(entry.card);
                                    let abbreviation = revenue.seat.abbreviation();
                                    let amount = format!("+${}", entry.amount);
                                    let id = controls::button(
                                        column,
                                        SeatName::Component(format!(
                                            "revenue-{}-{}",
                                            abbreviation.to_lowercase(),
                                            entry.card.instance
                                        )),
                                        format!(
                                            "Inspect {abbreviation} {} ({amount})",
                                            player.name
                                        ),
                                        Input::Inspect(target),
                                        if selection == Some(target) {
                                            READ_HIGHLIGHT
                                        } else {
                                            LEDGER
                                        },
                                        Node {
                                            min_height: vh(4.8),
                                            padding: UiRect::horizontal(vw(0.3)),
                                            border: UiRect::bottom(px(1.)),
                                            justify_content: JustifyContent::SpaceBetween,
                                            ..horizontal(0.35)
                                        },
                                    );
                                    column
                                        .commands()
                                        .entity(id)
                                        .insert((ReadingControl, BorderColor::all(RULE)))
                                        .with_children(|entry| {
                                            text(entry, player.name, 2.0, CREAM);
                                            text(entry, amount, 2.0, GOLD);
                                        });
                                }
                            });
                    }
                });
        });
}

fn purchase_order(root: &mut ChildSpawnerCommands, view: &Observation) {
    root.spawn(Node {
        justify_content: JustifyContent::SpaceBetween,
        ..horizontal(1.2)
    })
    .with_children(|footer| {
        let chooser = view.order_chooser.abbreviation();
        let reason = if view.revenue[0].total == view.revenue[1].total {
            format!("Revenue is tied. {chooser} lost the mini-game and chooses who buys first.")
        } else {
            format!("{chooser} earned less and chooses who buys first.")
        };
        text(footer, reason, 2.5, CREAM);
        if view.decision == Some(Decision::ChooseOrder) {
            footer.spawn(horizontal(1.25)).with_children(|choices| {
                for first in [Seat::SanFrancisco, Seat::NewYork] {
                    let label = format!("{} buys first", first.abbreviation());
                    let id = controls::button(
                        choices,
                        SeatName::Choice(format!(
                            "buy-first-{}",
                            first.abbreviation().to_lowercase()
                        )),
                        &label,
                        Input::Submit(view.submission(Action::ChooseOrder { first })),
                        GOLD,
                        Node {
                            padding: UiRect::axes(vw(1.05), vh(1.1)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                    );
                    choices
                        .commands()
                        .entity(id)
                        .with_children(|choice| text(choice, label, 2.5, INK));
                }
            });
        }
    });
}

fn inspection(
    root: &mut ChildSpawnerCommands,
    view: &Observation,
    prints: &Prints,
    selection: Option<Selection>,
) {
    let (player, amount) = match selection {
        Some(Selection::Offer { slot, card }) => {
            let offer = &view.offers[usize::from(slot - 1)];
            if offer.player.card != card {
                return;
            }
            (offer.player, format!("${} to sign", offer.cost))
        }
        _ => {
            let Some(player) = inspected_contributor(view, selection) else {
                return;
            };
            (player, format!("${} earned", player.revenue))
        }
    };
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: vh(10.),
            right: vw(1.5),
            align_items: AlignItems::Center,
            padding: UiRect::axes(vw(0.8), vh(0.8)),
            border_radius: BorderRadius::all(vh(0.75)),
            ..vertical(0.6)
        },
        GlobalZIndex(1),
        BackgroundColor(PAPER),
    ))
    .with_children(|panel| {
        text(panel, format!("{} / {amount}", player.name), 2.0, INK);
        face(panel, prints.front(player.card.identity), 29., 21.);
        reveal_button(
            panel,
            "close-inspection",
            "Close face".into(),
            Input::Close,
            CREAM,
        );
    });
}

pub(super) fn draw(
    root: &mut ChildSpawnerCommands,
    view: &Observation,
    prints: &Prints,
    selection: Option<Selection>,
    error: Option<&str>,
) {
    root.spawn((
        Node {
            width: percent(100.),
            height: percent(100.),
            padding: UiRect::axes(vw(1.8), vh(2.0)),
            ..vertical(1.65)
        },
        BackgroundColor(DARK),
    ))
    .with_children(|page| {
        page.spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            ..horizontal(1.0)
        })
        .with_children(|heading| {
            heading.spawn(vertical(0.2)).with_children(|heading| {
                text(heading, "Expert / Game 1 Buy Round", 1.8, FIELD_LABEL);
                text(heading, "Who buys first?", 4.2, CREAM);
            });
            let chooser = club(view.order_chooser);
            let status = if view.seat == view.order_chooser {
                format!("Your decision / {chooser}")
            } else {
                format!("{} manager / Waiting for {chooser}", club(view.seat))
            };
            text(heading, status, 2.6, GOLD);
        });
        market(page, view, prints);
        page.spawn(Node {
            min_height: vh(28.),
            ..horizontal(2.25)
        })
        .with_children(|ledgers| {
            for revenue in &view.revenue {
                ledger(ledgers, revenue, selection);
            }
        });
        purchase_order(page, view);
        if let Some(error) = error {
            text(page, error, 2.0, GOLD);
        }
        inspection(page, view, prints, selection);
    });
}
