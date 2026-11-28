//! Press Box market geometry and input. Signing authority stays with the series.
use super::{
    Prints, Session, card_help,
    controls::{
        self, CREAM, DARK, FIELD_LABEL, GOLD, INK, Input, Interface, PAPER, Selection, picture,
        row, text,
    },
};
use baseball_highlights_unfolded::{
    Action, Card, Decision, Observation, Offer, Player, Roster, Seat,
};
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
enum Source {
    InPlay(u8, Card),
    Offer(u8, Card),
    Finish,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Target {
    revision: u64,
    seat: Seat,
    source: Source,
}

impl Target {
    fn new(view: &Observation, source: Source) -> Self {
        Self {
            revision: view.revision,
            seat: view.seat,
            source,
        }
    }

    fn matches(self, view: &Observation) -> bool {
        self == Self::new(view, self.source)
    }

    fn offer(self, view: &Observation) -> Option<&Offer> {
        let Source::Offer(slot, card) = self.source else {
            return None;
        };
        let offer = &view.offers[usize::from(slot - 1)];
        (self.matches(view) && card == offer.player.card).then_some(offer)
    }

    fn in_play(self, view: &Observation) -> Option<Player> {
        let Source::InPlay(index, card) = self.source else {
            return None;
        };
        let player = buying_roster(view)?.in_play.get(usize::from(index))?;
        (self.matches(view) && view.demotion_pending && card == player.card).then_some(*player)
    }

    fn order(self) -> (u8, u8) {
        match self.source {
            Source::InPlay(index, _) => (0, index),
            Source::Offer(slot, _) => (1, slot),
            Source::Finish => (2, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Part {
    Face,
    Help,
    Action,
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
    pointer_ready: bool,
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
        let target = *self.hover.visible()?;
        if let Some(offer) = target.offer(view) {
            return Some(json!({
                "slot": offer.slot,
                "name": offer.player.name,
                "symbols": card_help::symbols(offer.player).collect::<Vec<_>>(),
                "budget": budget(offer, view.balance),
            }));
        }
        let player = target.in_play(view)?;
        Some(json!({
            "name": player.name,
            "symbols": card_help::symbols(player).collect::<Vec<_>>(),
            "zone": "In Play",
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
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) scale: f32,
    margin: f32,
    deck_width: f32,
    slot_width: f32,
    card_width: f32,
    market_y: f32,
    demotion: bool,
}

impl Layout {
    pub(super) fn new(size: Vec2, demotion: bool) -> Self {
        let scale = (size.x / 1080.).min(size.y / 720.);
        let margin = 23. * scale;
        let deck_width = if demotion { 113. } else { 88. } * scale;
        let slot_width = (size.x - 2. * margin - deck_width) / 6.;
        let card_width = (slot_width - 19. * scale).min(166. * scale);
        let mut layout = Self {
            width: size.x,
            height: size.y,
            scale,
            margin,
            deck_width,
            slot_width,
            card_width,
            market_y: 123. * scale,
            demotion,
        };
        if demotion {
            layout.card_width = (slot_width - 22. * scale).min(167. * scale) * 0.64;
            layout.market_y = layout.in_play_face(0).max.y + 65. * scale;
        }
        layout
    }

    pub(super) fn face(self, slot: u8) -> Rect {
        let x = self.margin + self.deck_width + (f32::from(slot) - 0.5) * self.slot_width
            - self.card_width / 2.;
        face_rect(x, self.market_y, self.card_width)
    }

    pub(super) fn in_play_face(self, index: u8) -> Rect {
        let stride = (self.width - 2. * self.margin) / 6.;
        let width = (stride - 21. * self.scale).min(166. * self.scale);
        let x = self.margin + (f32::from(index) + 0.5) * stride - width / 2.;
        face_rect(x, 117. * self.scale, width)
    }

    pub(super) fn deck_top(self) -> Rect {
        let height = if self.demotion { 69. } else { 91. } * self.scale;
        let width = height / 1.393;
        face_rect(
            self.margin + (self.deck_width - width - 5. * self.scale) / 2.,
            self.face(1).max.y - height - 24. * self.scale,
            width,
        )
    }

    fn club_top(self) -> f32 {
        self.face(1).max.y + if self.demotion { 35. } else { 95. } * self.scale
    }

    fn pile_height(self) -> f32 {
        (if self.demotion { 38. } else { 95. }) * self.scale
    }

    pub(super) fn club_pile(self, seat: Seat, pile: u8) -> Rect {
        let club_width = (self.width - 2. * self.margin - 14. * self.scale) / 2.;
        let club_x = self.margin
            + if seat == Seat::NewYork {
                club_width + 14. * self.scale
            } else {
                0.
            };
        let padding = if self.demotion { 5. } else { 11. } * self.scale;
        let gap = 5. * self.scale;
        let cell = (club_width - 2. * padding - 3. * gap) / 4.;
        let width = self.pile_height() / 1.393;
        let x = club_x
            + padding
            + (cell + gap) * f32::from(pile)
            + (cell - width - 5. * self.scale) / 2.;
        let y = self.club_top() + padding + if self.demotion { 20. } else { 31. } * self.scale;
        face_rect(x, y, width)
    }
}

fn face_rect(x: f32, y: f32, width: f32) -> Rect {
    Rect::from_corners(Vec2::new(x, y), Vec2::new(x + width, y + width * 1.393))
}

pub(super) fn place(x: f32, y: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(x),
        top: px(y),
        width: px(width),
        height: px(height),
        ..default()
    }
}

fn buying_roster(view: &Observation) -> Option<&Roster> {
    view.rosters
        .iter()
        .find(|roster| view.buyer == Some(roster.seat))
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

#[derive(Component)]
pub(super) struct OfferFace(pub(super) Card);

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
        target: Target::new(view, Source::Offer(offer.slot, offer.player.card)),
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
        .insert((
            region,
            BorderColor::all(CARD_FRAME),
            OfferFace(offer.player.card),
        ))
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
    rect: Rect,
    scale: f32,
    concealed: Option<super::market_transfer::Zone>,
) {
    let width = rect.width();
    let height = rect.height();
    parent
        .spawn((
            place(
                rect.min.x,
                rect.min.y,
                width + 5. * scale,
                height + 5. * scale,
            ),
            Pickable::IGNORE,
        ))
        .with_children(|stack| {
            if let Some(face) = face {
                let layers = count.saturating_sub(1).min(3);
                for layer in (1..=layers).rev() {
                    let offset = layer as f32 * 1.7 * scale;
                    panel(stack, place(offset, offset, width, height), CARD_FRAME);
                }
                let mut top = stack.spawn((
                    place(0., 0., width, height),
                    ImageNode::new(face),
                    Pickable::IGNORE,
                ));
                if let Some(zone) = concealed {
                    top.insert(super::market_transfer::PileTop { zone, count });
                }
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
    parent
        .spawn((
            Node {
                justify_content: JustifyContent::Center,
                ..place(
                    rect.center().x - 60. * scale,
                    rect.max.y + 4. * scale,
                    120. * scale,
                    18. * scale,
                )
            },
            Pickable::IGNORE,
        ))
        .with_children(|label| text(label, format!("{name}  {count}"), 13. * scale, CREAM));
}

fn club(
    root: &mut ChildSpawnerCommands,
    view: &Observation,
    roster: &Roster,
    layout: Layout,
    prints: &Prints,
    x: f32,
    width: f32,
) {
    use super::market_transfer::Zone;

    let s = layout.scale;
    let top = layout.club_top();
    let height = layout.height - top - 41. * s;
    let compact = layout.demotion;
    let club = panel(
        root,
        Node {
            padding: UiRect::all(px(if compact { 5. * s } else { 11. * s })),
            border_radius: BorderRadius::all(px(6. * s)),
            ..place(x, top, width, height)
        },
        CLUB_PANEL,
    );
    root.commands().entity(club).with_children(|club| {
        club.spawn(Node {
            flex_grow: 1.,
            justify_content: JustifyContent::SpaceBetween,
            ..row(5. * s)
        })
        .with_children(|head| {
            text(
                head,
                club_name(roster.seat),
                if compact { 15. } else { 19. } * s,
                CREAM,
            );
            text(
                head,
                format!(
                    "{} active   ·   ${} left",
                    roster.active,
                    view.balances[usize::from(roster.seat == Seat::NewYork)]
                ),
                if compact { 14. } else { 16. } * s,
                FIELD_LABEL,
            );
        });
    });
    let piles = [
        (
            "Lineup",
            roster.lineup,
            (roster.lineup > 0).then(|| prints.back.clone()),
            Some(Zone::Lineup(roster.seat)),
        ),
        (
            "In Play",
            roster.in_play.len(),
            roster
                .in_play
                .last()
                .map(|player| prints.front(player.card.identity)),
            None,
        ),
        (
            "Dugout",
            roster.dugout.len(),
            roster
                .dugout
                .last()
                .map(|player| prints.front(player.card.identity)),
            None,
        ),
        (
            "Minor Leagues",
            roster.minor_leagues,
            (roster.minor_leagues > 0).then(|| prints.back.clone()),
            Some(Zone::Minors(roster.seat)),
        ),
    ];
    for (index, (name, count, face, concealed)) in piles.into_iter().enumerate() {
        pile(
            root,
            name,
            count,
            face,
            layout.club_pile(roster.seat, index as u8),
            s,
            concealed,
        );
    }
}

fn in_play_card(
    root: &mut ChildSpawnerCommands,
    view: &Observation,
    layout: Layout,
    index: u8,
    player: Player,
    prints: &Prints,
) {
    let s = layout.scale;
    let face = layout.in_play_face(index);
    let region = Region {
        target: Target::new(view, Source::InPlay(index, player.card)),
        part: Part::Face,
    };
    let card = controls::button(
        root,
        SeatName::Component(format!("inspect-{}", player.card.identity)),
        format!("Inspect {}", player.name),
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
                prints.front(player.card.identity),
                face.width() - 6. * s,
                face.height() - 6. * s,
            );
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
                let prompt = match (view.seat == buyer, view.demotion_pending) {
                    (true, true) => "Choose a player to send to the Minor Leagues".to_owned(),
                    (true, false) => "Buy Row".to_owned(),
                    (false, true) => {
                        format!("Waiting for {} to choose a demotion", club_name(buyer))
                    }
                    (false, false) => format!("Waiting for {} to buy", club_name(buyer)),
                };
                text(
                    title,
                    prompt,
                    if view.demotion_pending { 21. } else { 25. } * s,
                    CREAM,
                );
                let funds = format!("{}   ${} left", buyer.abbreviation(), view.balance);
                text(title, funds, 20. * s, GOLD);
            });
        heading.spawn(row(9. * s)).with_children(|options| {
            if matches!(view.decision, Some(Decision::Sign { .. })) {
                let region = Region {
                    target: Target::new(view, Source::Finish),
                    part: Part::Action,
                };
                let pass = action_button(
                    options,
                    SeatName::Choice("pass".into()),
                    "Finish buying",
                    Input::Submit(view.submission(Action::Pass)),
                    s,
                );
                options.commands().entity(pass).insert((
                    region,
                    BorderColor::all(PAPER),
                    Node {
                        padding: UiRect::axes(px(12. * s), px(6. * s)),
                        align_items: AlignItems::Center,
                        border: UiRect::all(px(s)),
                        ..default()
                    },
                ));
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

    if view.demotion_pending
        && let Some(roster) = buying_roster(view)
    {
        for (index, &player) in roster.in_play.iter().enumerate() {
            in_play_card(root, view, layout, index as u8, player, prints);
        }
        root.spawn((
            place(layout.margin, layout.market_y - 25. * s, 110. * s, 21. * s),
            Pickable::IGNORE,
        ))
        .with_children(|heading| text(heading, "Buy Row", 17. * s, CREAM));
    }
    pile(
        root,
        "Free Agent Deck",
        view.free_agent_deck,
        (view.free_agent_deck > 0).then(|| prints.back.clone()),
        layout.deck_top(),
        s,
        None,
    );
    for offer in &view.offers {
        offer_card(root, view, layout, offer, prints);
    }
    let club_width = (outer_width - 14. * s) / 2.;
    for (index, roster) in view.rosters.iter().enumerate() {
        let x = layout.margin + index as f32 * (club_width + 14. * s);
        club(root, view, roster, layout, prints, x, club_width);
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
            if matches!(view.decision, Some(Decision::Sign { ref affordable }) if affordable.is_empty())
                && let Some(cheapest) = view.offers.iter().map(|offer| offer.cost).min()
            {
                text(
                    status,
                    format!("   Lowest price ${cheapest}   ·   ${} left", view.balance),
                    15. * s,
                    GOLD,
                );
            }
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
) -> Entity {
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
    button
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
    order.sort_by_key(|region| (region.target.order(), region.part));
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
            pointer_ready: interaction.revision.is_none(),
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
        interaction.pointer_ready = true;
        interaction.keyboard = false;
    }
    let pointed = regions
        .iter()
        .filter(|(region, hovered, _)| {
            region.target.revision == revision
                && hovered.get()
                && cursor.is_some()
                && interaction.pointer_ready
                && region.target.source != Source::Finish
        })
        .max_by_key(|(region, _, _)| region.part != Part::Gap)
        .map(|(region, _, _)| *region);
    let clicked = interaction.clicked.take();
    if let Some(region) = clicked {
        interaction.focus = Some(region);
        interaction.pointer_ready = true;
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
                if let Some(focus) = &mut interaction.focus
                    && focus.target.source != Source::Finish
                {
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
        interaction
            .focus
            .filter(|region| region.target.source != Source::Finish)
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
    layout: Layout,
    prints: &Prints,
) {
    let s = layout.scale;
    let offer = target.offer(view);
    let (face, player, help_id) = match (target.source, offer, target.in_play(view)) {
        (Source::Offer(slot, _), Some(offer), _) => (
            layout.face(slot),
            offer.player,
            format!("offer-help-{slot}"),
        ),
        (Source::InPlay(index, card), _, Some(player)) => (
            layout.in_play_face(index),
            player,
            format!("in-play-help-{}", card.identity),
        ),
        _ => return,
    };
    let help_width = 334. * s;
    let gap = 12. * s;
    let help_right = face.max.x + gap + help_width <= layout.width - layout.margin;
    let help_x = if help_right {
        face.max.x + gap
    } else {
        face.min.x - gap - help_width
    };
    let action_y = face.max.y + if offer.is_some() { 53. } else { 14. } * s;
    let help_height = if player.pinch_hitter { 153. } else { 83. } * s;
    let help_y = (face.max.y + if offer.is_some() { 29. } else { 14. } * s)
        .min(layout.height - help_height - 20. * s);
    let region = Region {
        target,
        part: Part::Help,
    };
    let help = controls::button(
        root,
        SeatName::Component(help_id),
        format!("{}'s card symbols", player.name),
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
        .with_children(|panel| card_help::draw(panel, prints, player, s));

    let edge = if help_right {
        face.max.x
    } else {
        face.min.x - gap
    };
    bridge(root, target, place(edge, help_y, gap, 61. * s));
    bridge(
        root,
        target,
        place(face.min.x, face.max.y, face.width(), action_y - face.max.y),
    );

    let action = if let Some(offer) = offer {
        root.spawn((
            Node {
                justify_content: JustifyContent::Center,
                ..place(face.min.x, face.max.y + 26. * s, face.width(), 26. * s)
            },
            Pickable::IGNORE,
        ))
        .with_children(|amount| text(amount, budget(offer, view.balance), 13. * s, GOLD));
        can_sign(view, offer).then(|| {
            (
                format!("sign-{}-{}", offer.slot, player.card.instance),
                format!("Sign {} for ${}", player.name, offer.cost),
                Action::Sign {
                    slot: offer.slot,
                    card: player.card,
                },
                "Sign",
                prints.autograph.clone(),
                18. * s,
            )
        })
    } else {
        let card = player.card;
        matches!(&view.decision, Some(Decision::Demote { eligible }) if eligible.contains(&card))
            .then(|| {
                (
                    format!("demote-{}-{}", card.identity, card.instance),
                    format!("Send {} to Minors", player.name),
                    Action::Demote { card },
                    "Send to Minors",
                    prints.transfer_card.clone(),
                    13. * s,
                )
            })
    };

    if let Some((choice, label, action, words, icon, text_size)) = action {
        let region = Region {
            target,
            part: Part::Action,
        };
        let button = controls::button(
            root,
            SeatName::Choice(choice),
            label,
            Input::Submit(view.submission(action)),
            GOLD,
            Node {
                border: UiRect::all(px(2. * s)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(6. * s),
                ..place(face.min.x, action_y, face.width(), 34. * s)
            },
        );
        root.commands()
            .entity(button)
            .insert((region, BorderColor::all(GOLD)))
            .with_children(|action| {
                picture(action, icon, 27. * s, 27. * s);
                text(action, words, text_size, INK);
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
            commands.entity(root).with_children(|root| {
                root.spawn((
                    Disclosure,
                    place(0., 0., layout.width, layout.height),
                    ZIndex(2),
                    Pickable::IGNORE,
                ))
                .with_children(|layer| disclosure(layer, &view, target, *layout, &prints));
            });
        }
    }

    for (_, region, mut border) in &mut existing {
        let color = if Some(*region) == interaction.focus {
            FOCUS
        } else {
            match region.part {
                Part::Face if Some(region.target) == visible => GOLD,
                Part::Face => CARD_FRAME,
                Part::Action if region.target.source == Source::Finish => PAPER,
                Part::Action => GOLD,
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
