//! Press Box market geometry and input. Signing authority stays with the series.
use super::{
    Prints, Session, card_help,
    controls::{
        self, CREAM, DARK, FIELD_LABEL, GOLD, INK, Input, Interface, PAPER, Selection, col,
        picture, row, text,
    },
};
use baseball_highlights_unfolded::{Action, Card, Decision, Observation, Offer, Roster, Seat};
use bevy::{
    input::keyboard::{Key, KeyboardFocusLost, KeyboardInput},
    picking::{hover::Hovered, prelude::Pickable},
    prelude::*,
    window::PrimaryWindow,
};
use serde_json::{Value, json};
use std::time::Duration;
use unfolded_hover::Hover;
use unfolded_tabletop::seat::{self, Name as SeatName};

const BACKGROUND: Color = Color::srgb_u8(21, 40, 47);
const CLUB_PANEL: Color = Color::srgb_u8(36, 60, 64);
const CARD_FRAME: Color = Color::srgb_u8(81, 107, 101);
const FOCUS: Color = Color::srgb_u8(255, 242, 205);
const DWELL: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Target {
    revision: u64,
    seat: Seat,
    slot: u8,
    card: Card,
}

impl Target {
    fn new(view: &Observation, offer: &Offer) -> Self {
        Self {
            revision: view.revision,
            seat: view.seat,
            slot: offer.slot,
            card: offer.player.card,
        }
    }

    fn offer(self, view: &Observation) -> Option<&Offer> {
        let offer = &view.offers[usize::from(self.slot - 1)];
        (self == Self::new(view, offer)).then_some(offer)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Part {
    Face,
    Help,
    Sign,
    Gap,
}

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(super) struct Region {
    pub(super) target: Target,
    part: Part,
}

#[derive(Resource, Default)]
pub(super) struct Interaction {
    hover: Hover<Target>,
    focus: Option<Region>,
    cursor: Option<Vec2>,
    keyboard: bool,
    clicked: Option<Region>,
    revision: Option<u64>,
}

impl Interaction {
    pub(super) fn click(&mut self, region: Region) {
        self.clicked = Some(region);
    }

    pub(super) fn waiting(&self) -> bool {
        self.hover.waiting()
    }

    pub(super) fn focus(&self) -> Option<Region> {
        self.focus
    }

    pub(super) fn reading(&self, view: &Observation) -> Option<Value> {
        let offer = self.hover.visible()?.offer(view)?;
        Some(json!({
            "slot": offer.slot,
            "name": offer.player.name,
            "symbols": card_help::symbols(offer.player).collect::<Vec<_>>(),
            "budget": budget(offer, view.balance),
        }))
    }
}

pub(super) static KEYS: [seat::SeatKey; 5] = [
    key("Tab", KeyCode::Tab, Key::Tab, false),
    // The seat protocol sends one key. This logical key represents reverse Tab;
    // the desktop path receives a physical Tab with Shift held.
    key("Shift+Tab", KeyCode::Tab, Key::NavigatePrevious, false),
    key("Escape", KeyCode::Escape, Key::Escape, false),
    key("Enter", KeyCode::Enter, Key::Enter, true),
    key("Space", KeyCode::Space, Key::Space, false),
];

const fn key(name: &'static str, code: KeyCode, logical: Key, commits: bool) -> seat::SeatKey {
    seat::SeatKey {
        name,
        code,
        logical: seat::Logical::Named(logical),
        commits,
    }
}

#[derive(Clone, Copy, Component)]
pub(super) struct Layout {
    width: f32,
    height: f32,
    scale: f32,
    margin: f32,
    deck_width: f32,
    slot_width: f32,
    card_width: f32,
    card_height: f32,
}

impl Layout {
    pub(super) fn new(size: Vec2) -> Self {
        let scale = (size.x / 1080.).min(size.y / 720.);
        let margin = 23. * scale;
        let deck_width = 88. * scale;
        let slot_width = (size.x - 2. * margin - deck_width) / 6.;
        let card_width = (slot_width - 19. * scale).min(166. * scale);
        Self {
            width: size.x,
            height: size.y,
            scale,
            margin,
            deck_width,
            slot_width,
            card_width,
            card_height: card_width * 1.393,
        }
    }

    fn face(self, slot: u8) -> Rect {
        let x = self.margin + self.deck_width + (f32::from(slot) - 0.5) * self.slot_width
            - self.card_width / 2.;
        let y = 123. * self.scale;
        Rect::from_corners(
            Vec2::new(x, y),
            Vec2::new(x + self.card_width, y + self.card_height),
        )
    }

    fn club_top(self) -> f32 {
        self.face(1).max.y + 95. * self.scale
    }
}

fn place(x: f32, y: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(x),
        top: px(y),
        width: px(width),
        height: px(height),
        ..default()
    }
}

fn panel(parent: &mut ChildSpawnerCommands, node: Node, color: Color) -> Entity {
    parent
        .spawn((node, BackgroundColor(color), Pickable::IGNORE))
        .id()
}

fn club_name(seat: Seat) -> &'static str {
    match seat {
        Seat::SanFrancisco => "San Francisco",
        Seat::NewYork => "New York",
    }
}

fn budget(offer: &Offer, balance: u8) -> String {
    if let Some(left) = offer.balance_after {
        format!("${balance} − ${} = ${left} left", offer.cost)
    } else {
        format!("${} short", offer.cost.saturating_sub(balance))
    }
}

fn can_sign(view: &Observation, offer: &Offer) -> bool {
    matches!(&view.decision, Some(Decision::Sign { affordable }) if affordable.contains(&offer.player.card))
}

fn offer_card(
    root: &mut ChildSpawnerCommands,
    view: &Observation,
    layout: Layout,
    offer: &Offer,
    prints: &Prints,
) {
    let s = layout.scale;
    let face = layout.face(offer.slot);
    let region = Region {
        target: Target::new(view, offer),
        part: Part::Face,
    };
    let card = controls::button(
        root,
        SeatName::Component(format!("inspect-slot-{}", offer.slot)),
        format!("Inspect {}, ${}", offer.player.name, offer.cost),
        Input::ReadOffer(region),
        BACKGROUND,
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2. * s)),
            ..place(face.min.x, face.min.y, face.width(), face.height())
        },
    );
    root.commands()
        .entity(card)
        .insert((region, BorderColor::all(CARD_FRAME)))
        .with_children(|image| {
            picture(
                image,
                prints.front(offer.player.card.identity),
                face.width() - 6. * s,
                face.height() - 6. * s,
            );
        });
    root.spawn((
        Node {
            justify_content: JustifyContent::Center,
            ..place(face.min.x, face.max.y + 4. * s, face.width(), 25. * s)
        },
        Pickable::IGNORE,
    ))
    .with_children(|price| text(price, format!("${}", offer.cost), 19. * s, CREAM));
}

fn pile(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    count: usize,
    face: Option<Handle<Image>>,
    scale: f32,
    height: f32,
) {
    let width = height / 1.393;
    parent
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                flex_grow: 1.,
                ..col(5. * scale)
            },
            Pickable::IGNORE,
        ))
        .with_children(|pile| {
            pile.spawn((
                Node {
                    width: px(width + 5. * scale),
                    height: px(height + 5. * scale),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|stack| {
                if let Some(face) = face {
                    let layers = count.saturating_sub(1).min(3);
                    for layer in (1..=layers).rev() {
                        let offset = layer as f32 * 1.7 * scale;
                        panel(stack, place(offset, offset, width, height), CARD_FRAME);
                    }
                    picture(stack, face, width, height);
                } else {
                    stack.spawn((
                        Node {
                            border: UiRect::all(px(scale)),
                            border_radius: BorderRadius::all(px(4. * scale)),
                            ..place(0., 0., width, height)
                        },
                        BorderColor::all(CARD_FRAME),
                        Pickable::IGNORE,
                    ));
                }
            });
            text(pile, format!("{name}  {count}"), 14. * scale, CREAM);
        });
}

fn club(
    root: &mut ChildSpawnerCommands,
    view: &Observation,
    roster: &Roster,
    layout: Layout,
    prints: &Prints,
    x: f32,
    width: f32,
    selection: Option<Selection>,
) {
    let s = layout.scale;
    let top = layout.club_top();
    let height = layout.height - top - 49. * s;
    let card_height = if view.demotion_pending { 63. } else { 95. } * s;
    let club = panel(
        root,
        Node {
            padding: UiRect::all(px(11. * s)),
            row_gap: px(7. * s),
            flex_direction: FlexDirection::Column,
            border_radius: BorderRadius::all(px(6. * s)),
            ..place(x, top, width, height)
        },
        CLUB_PANEL,
    );
    root.commands().entity(club).with_children(|club| {
        club.spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            ..row(5. * s)
        })
        .with_children(|head| {
            text(head, club_name(roster.seat), 19. * s, CREAM);
            text(
                head,
                format!(
                    "{} active  /  ${} left",
                    roster.active,
                    view.balances[usize::from(roster.seat == Seat::NewYork)]
                ),
                16. * s,
                FIELD_LABEL,
            );
        });
        club.spawn(row(5. * s)).with_children(|piles| {
            pile(
                piles,
                "Lineup",
                roster.lineup,
                (roster.lineup > 0).then(|| prints.back.clone()),
                s,
                card_height,
            );
            pile(
                piles,
                "In Play",
                roster.in_play.len(),
                roster
                    .in_play
                    .last()
                    .map(|player| prints.front(player.card.identity)),
                s,
                card_height,
            );
            pile(
                piles,
                "Dugout",
                roster.dugout.len(),
                roster
                    .dugout
                    .last()
                    .map(|player| prints.front(player.card.identity)),
                s,
                card_height,
            );
            pile(
                piles,
                "Minor Leagues",
                roster.minor_leagues,
                (roster.minor_leagues > 0).then(|| prints.back.clone()),
                s,
                card_height,
            );
        });
        // The current demotion route remains available while the dedicated In Play
        // layout is developed. The market slots do not move when a demotion is due.
        if view.buyer == Some(roster.seat) && view.demotion_pending {
            for (row, players) in roster.in_play.chunks(3).enumerate() {
                controls::card_rail(
                    club,
                    players,
                    prints,
                    selection,
                    Selection::PublicCard,
                    1 + 3 * row,
                );
            }
        }
    });
}

pub(super) fn draw(
    root: &mut ChildSpawnerCommands,
    layout: Layout,
    view: &Observation,
    prints: &Prints,
    ui: &Interface,
) {
    let s = layout.scale;
    let outer_width = layout.width - 2. * layout.margin;
    let buyer = view.buyer.expect("the active Buy Round has a buyer");
    let header = panel(
        root,
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(px(layout.margin)),
            ..place(0., 0., layout.width, 55. * s)
        },
        DARK,
    );
    root.commands().entity(header).with_children(|head| {
        text(head, "Expert / Game 1 Buy Round", 23. * s, CREAM);
        text(
            head,
            format!("Viewing {}", club_name(view.seat)),
            18. * s,
            FIELD_LABEL,
        );
    });
    root.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..place(layout.margin, 63. * s, outer_width, 49. * s)
        },
        Pickable::IGNORE,
    ))
    .with_children(|heading| {
        heading
            .spawn(Node {
                align_items: AlignItems::Center,
                ..row(20. * s)
            })
            .with_children(|title| {
                text(title, "Buy Row", 25. * s, CREAM);
                let decision = if view.demotion_pending {
                    "chooses a demotion"
                } else {
                    "is buying"
                };
                text(
                    title,
                    format!("{} {decision}   ${} left", club_name(buyer), view.balance),
                    21. * s,
                    GOLD,
                );
            });
        heading.spawn(row(9. * s)).with_children(|options| {
            if matches!(view.decision, Some(Decision::Sign { .. })) {
                action_button(
                    options,
                    SeatName::Choice("pass".into()),
                    "Finish buying",
                    Input::Submit(view.submission(Action::Pass)),
                    s,
                );
            }
            action_button(
                options,
                SeatName::Component("round-history".into()),
                "Round history",
                Input::Inspect(Selection::History),
                s,
            );
        });
    });

    root.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::End,
            ..place(
                layout.margin,
                123. * s,
                layout.deck_width,
                layout.card_height,
            )
        },
        Pickable::IGNORE,
    ))
    .with_children(|deck| {
        pile(
            deck,
            "Free Agent Deck",
            view.free_agent_deck,
            (view.free_agent_deck > 0).then(|| prints.back.clone()),
            s,
            91. * s,
        );
    });
    for offer in &view.offers {
        offer_card(root, view, layout, offer, prints);
    }
    let club_width = (outer_width - 14. * s) / 2.;
    for (index, roster) in view.rosters.iter().enumerate() {
        let x = layout.margin + index as f32 * (club_width + 14. * s);
        club(
            root,
            view,
            roster,
            layout,
            prints,
            x,
            club_width,
            ui.selection,
        );
    }
    root.spawn((
        place(layout.margin, layout.height - 34. * s, outer_width, 26. * s),
        Pickable::IGNORE,
    ))
    .with_children(|status| {
        if let Some(error) = &ui.error {
            text(status, error, 16. * s, GOLD);
        } else if let Some(&event) = view.events.last() {
            text(status, controls::event_text(event), 15. * s, FIELD_LABEL);
        }
    });
    if let Some(selection) = ui.selection {
        let top = if selection == Selection::History {
            63. * s
        } else {
            layout.club_top()
        };
        let width = 435. * s;
        root.spawn((
            place(
                layout.width - width - layout.margin,
                top,
                width,
                layout.height - top - 48. * s,
            ),
            ZIndex(3),
            Pickable::IGNORE,
        ))
        .with_children(|overlay| controls::inspector(overlay, view, prints, ui.selection, s));
    }
}

fn action_button(
    parent: &mut ChildSpawnerCommands,
    name: SeatName,
    label: &str,
    input: Input,
    s: f32,
) {
    let button = controls::button(
        parent,
        name,
        label,
        input,
        PAPER,
        Node {
            padding: UiRect::axes(px(13. * s), px(7. * s)),
            align_items: AlignItems::Center,
            ..default()
        },
    );
    parent
        .commands()
        .entity(button)
        .with_children(|button| text(button, label, 15. * s, INK));
}

fn move_focus(
    interaction: &mut Interaction,
    controls: &Query<(&Region, &Hovered, Option<&controls::Control>)>,
    reverse: bool,
) {
    let mut order: Vec<_> = controls
        .iter()
        .filter_map(|(region, _, control)| control.is_some().then_some(*region))
        .collect();
    order.sort_by_key(|region| (region.target.slot, region.part));
    if reverse {
        order.reverse();
    }
    interaction.focus = interaction
        .focus
        .and_then(|focus| order.iter().position(|region| *region == focus))
        .and_then(|index| order.get(index + 1).copied())
        .or_else(|| order.first().copied());
    interaction.keyboard = true;
}

pub(super) fn interact(
    session: ResMut<Session>,
    mut ui: ResMut<Interface>,
    mut interaction: ResMut<Interaction>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut shifts: Local<[bool; 2]>,
    mut keys: MessageReader<KeyboardInput>,
    mut focus_lost: MessageReader<KeyboardFocusLost>,
    layout: Query<(), With<Layout>>,
    regions: Query<(&Region, &Hovered, Option<&controls::Control>)>,
) {
    let mut session = session;
    let active = layout.single().is_ok() && ui.selection.is_none();
    let revision = session.game.revision();
    let shown_revision = active.then_some(revision);
    if interaction.revision != shown_revision {
        *interaction = Interaction {
            cursor: window.cursor_position(),
            revision: shown_revision,
            ..default()
        };
    }
    if !focus_lost.is_empty() {
        focus_lost.clear();
        *shifts = [false; 2];
    }

    let cursor = window.cursor_position();
    if cursor != interaction.cursor {
        interaction.cursor = cursor;
        interaction.keyboard = false;
    }
    let pointed = regions
        .iter()
        .filter(|(region, hovered, _)| {
            region.target.revision == revision && hovered.get() && cursor.is_some()
        })
        .max_by_key(|(region, _, _)| region.part != Part::Gap)
        .map(|(region, _, _)| *region);
    let clicked = interaction.clicked.take();
    if let Some(region) = clicked {
        interaction.focus = Some(region);
        interaction.keyboard = false;
    }
    let mut escape = false;
    for event in keys.read() {
        // Read modifiers in event order. A quick Shift+Tab may press and
        // release Shift within a single rendered frame.
        if matches!(event.key_code, KeyCode::ShiftLeft | KeyCode::ShiftRight) {
            shifts[usize::from(event.key_code == KeyCode::ShiftRight)] = event.state.is_pressed();
            continue;
        }
        if !active || !event.state.is_pressed() || event.repeat {
            continue;
        }
        match event.key_code {
            KeyCode::Tab => {
                let reverse = event.logical_key == Key::NavigatePrevious || shifts.contains(&true);
                move_focus(&mut interaction, &regions, reverse);
            }
            KeyCode::Escape => {
                escape = true;
                if let Some(focus) = &mut interaction.focus {
                    focus.part = Part::Face;
                }
            }
            KeyCode::Enter | KeyCode::NumpadEnter => {
                interaction.keyboard = true;
                if let Some(action) = regions.iter().find_map(|(region, _, control)| {
                    if Some(*region) == interaction.focus
                        && let Some(controls::Control {
                            input: Input::Submit(action),
                            ..
                        }) = control
                    {
                        return Some(action.clone());
                    }
                    None
                }) {
                    controls::submit(&mut session, &mut ui, action);
                    return;
                }
            }
            _ => {}
        }
    }
    if !active {
        return;
    }
    let target = if interaction.keyboard {
        interaction.focus
    } else {
        clicked.or(pointed)
    };
    let dwell = if interaction.keyboard || clicked.is_some() {
        Duration::ZERO
    } else {
        DWELL
    };
    interaction.hover.update(
        target.map(|region| region.target),
        true,
        time.delta(),
        dwell,
    );
    if escape {
        interaction.hover.dismiss();
    }
}

#[derive(Component)]
pub(super) struct Disclosure;

fn bridge(root: &mut ChildSpawnerCommands, target: Target, node: Node) {
    root.spawn((
        node,
        Hovered::default(),
        Pickable::default(),
        Region {
            target,
            part: Part::Gap,
        },
    ));
}

fn disclosure(
    root: &mut ChildSpawnerCommands,
    view: &Observation,
    target: Target,
    offer: &Offer,
    layout: Layout,
    prints: &Prints,
) {
    let s = layout.scale;
    let face = layout.face(target.slot);
    let help_width = 334. * s;
    let gap = 12. * s;
    let help_right = face.max.x + gap + help_width <= layout.width - layout.margin;
    let help_x = if help_right {
        face.max.x + gap
    } else {
        face.min.x - gap - help_width
    };
    let help_y = face.max.y + 29. * s;
    let help_height = if offer.player.pinch_hitter { 153. } else { 83. } * s;
    let region = Region {
        target,
        part: Part::Help,
    };
    let help = controls::button(
        root,
        SeatName::Component(format!("offer-help-{}", offer.slot)),
        format!("{}'s card symbols", offer.player.name),
        Input::ReadOffer(region),
        CREAM,
        Node {
            border: UiRect::all(px(2. * s)),
            padding: UiRect::all(px(7. * s)),
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(5. * s),
            ..place(help_x, help_y, help_width, help_height)
        },
    );
    root.commands()
        .entity(help)
        .insert((region, BorderColor::all(CREAM)))
        .with_children(|panel| {
            card_help::draw(panel, prints, offer.player, s);
        });

    // The face, its explanation and its action share hover identity. These
    // transparent regions carry pointer travel and never submit a choice.
    let edge = if help_right {
        face.max.x
    } else {
        face.min.x - gap
    };
    bridge(root, target, place(edge, help_y, gap, 61. * s));
    bridge(
        root,
        target,
        place(face.min.x, face.max.y, face.width(), 53. * s),
    );

    root.spawn((
        Node {
            justify_content: JustifyContent::Center,
            ..place(face.min.x, face.max.y + 26. * s, face.width(), 26. * s)
        },
        Pickable::IGNORE,
    ))
    .with_children(|amount| {
        text(
            amount,
            budget(offer, view.balance).replace('−', "-"),
            13. * s,
            GOLD,
        )
    });
    if can_sign(view, offer) {
        let region = Region {
            target,
            part: Part::Sign,
        };
        let sign = controls::button(
            root,
            SeatName::Choice(format!("sign-{}-{}", target.slot, target.card.instance)),
            format!("Sign {} for ${}", offer.player.name, offer.cost),
            Input::Submit(view.submission(Action::Sign {
                slot: target.slot,
                card: target.card,
            })),
            GOLD,
            Node {
                border: UiRect::all(px(2. * s)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(6. * s),
                ..place(face.min.x, face.max.y + 53. * s, face.width(), 34. * s)
            },
        );
        root.commands()
            .entity(sign)
            .insert((region, BorderColor::all(GOLD)))
            .with_children(|action| {
                picture(action, prints.autograph.clone(), 27. * s, 27. * s);
                text(action, "Sign", 18. * s, INK);
            });
    }
}

pub(super) fn present(
    mut commands: Commands,
    layout: Query<(Entity, &Layout)>,
    interaction: Res<Interaction>,
    session: Res<Session>,
    prints: Res<Prints>,
    mut existing: Query<(Entity, &Region, &mut BorderColor)>,
    disclosures: Query<Entity, With<Disclosure>>,
    mut shown: Local<Option<(Entity, Target)>>,
) {
    let visible = interaction.hover.visible().copied();
    // The shared dwell clock can update every frame. Keep the board and actual
    // input entities intact unless the visible offer or the board changes.
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
            if let Some(offer) = target.offer(&view) {
                commands.entity(root).with_children(|root| {
                    root.spawn((
                        Disclosure,
                        place(0., 0., layout.width, layout.height),
                        ZIndex(2),
                        Pickable::IGNORE,
                    ))
                    .with_children(|layer| {
                        disclosure(layer, &view, target, offer, *layout, &prints)
                    });
                });
            }
        }
    }

    for (_, region, mut border) in &mut existing {
        let color = if Some(*region) == interaction.focus {
            FOCUS
        } else {
            match region.part {
                Part::Face if Some(region.target) == visible => GOLD,
                Part::Face => CARD_FRAME,
                Part::Sign => GOLD,
                Part::Help => CREAM,
                Part::Gap => continue,
            }
        };
        border.set_if_neq(BorderColor::all(color));
    }
}

pub(super) fn background() -> BackgroundColor {
    BackgroundColor(BACKGROUND)
}
