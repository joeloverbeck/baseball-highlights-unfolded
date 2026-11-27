use super::{FIELD, HEADER, Prints, Session};
use baseball_highlights_unfolded::{
    Action, Card, Decision, Event, Observation, Offer, Player, Submission,
};
use bevy::{
    picking::{hover::Hovered, prelude::Pickable},
    prelude::*,
    ui_widgets::{Activate, Button},
    window::PrimaryWindow,
};
use serde_json::json;
use unfolded_tabletop::seat::{self, Admission, Hit, Name as SeatName, Observed};

const INK: Color = Color::srgb(0.14, 0.10, 0.07);
const MUTED: Color = Color::srgb(0.34, 0.24, 0.15);
const CREAM: Color = Color::srgb(0.97, 0.91, 0.80);
const PAPER: Color = Color::srgb(0.92, 0.84, 0.69);
const GOLD: Color = Color::srgb(0.96, 0.78, 0.48);
const DARK: Color = Color::srgb(0.10, 0.16, 0.13);
const FIELD_LABEL: Color = Color::srgb(0.81, 0.88, 0.77);

#[derive(Clone, Copy, PartialEq)]
enum Selection {
    Offer(u8),
    InPlay(Card),
}

#[derive(Resource, Default)]
pub struct Interface {
    selection: Option<Selection>,
    error: Option<String>,
}

#[derive(Clone)]
enum Input {
    Inspect(Selection),
    Submit(Submission),
    Close,
}

#[derive(Component, Clone)]
pub(super) struct Control {
    name: SeatName,
    label: String,
    input: Input,
    color: Color,
}

#[derive(Component, Clone)]
pub struct Board {
    revision: u64,
}

fn press(
    event: On<Activate>,
    controls: Query<&Control>,
    mut session: ResMut<Session>,
    mut ui: ResMut<Interface>,
) {
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    match &control.input {
        Input::Inspect(selection) => {
            ui.selection = Some(*selection);
            ui.error = None;
        }
        Input::Close => {
            ui.selection = None;
            ui.error = None;
        }
        Input::Submit(submission) => match session.game.submit(submission.clone()) {
            Ok(()) => {
                ui.selection = match submission.action {
                    Action::Sign { slot, .. } => Some(Selection::Offer(slot)),
                    Action::Demote { .. } => None,
                };
                ui.error = None;
            }
            Err(error) => ui.error = Some(error.to_string()),
        },
    }
}

pub fn hover(mut buttons: Query<(&Hovered, &Control, &mut BackgroundColor), Changed<Hovered>>) {
    for (hovered, control, mut color) in &mut buttons {
        color.0 = if hovered.get() { GOLD } else { control.color };
    }
}

fn col(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(gap),
        ..default()
    }
}

fn row(gap: f32) -> Node {
    Node {
        column_gap: px(gap),
        ..default()
    }
}

fn text(parent: &mut ChildSpawnerCommands, words: impl Into<String>, size: f32, color: Color) {
    parent.spawn((
        Text::new(words),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    ));
}

fn picture(parent: &mut ChildSpawnerCommands, handle: Handle<Image>, width: f32, height: f32) {
    parent.spawn((
        ImageNode::new(handle),
        Node {
            width: px(width),
            height: px(height),
            flex_shrink: 0.,
            ..default()
        },
        Pickable::IGNORE,
    ));
}

fn button(
    parent: &mut ChildSpawnerCommands,
    name: SeatName,
    label: impl Into<String>,
    input: Input,
    color: Color,
    node: Node,
) -> Entity {
    parent
        .spawn((
            Button,
            Node {
                border_radius: BorderRadius::all(px(5.)),
                ..node
            },
            BackgroundColor(color),
            Hovered::default(),
            Control {
                name,
                label: label.into(),
                input,
                color,
            },
        ))
        .observe(press)
        .id()
}

fn short_button(
    parent: &mut ChildSpawnerCommands,
    name: SeatName,
    label: String,
    input: Input,
    color: Color,
) {
    let id = button(
        parent,
        name,
        label.clone(),
        input,
        color,
        Node {
            padding: UiRect::axes(px(12.), px(8.)),
            justify_content: JustifyContent::Center,
            ..default()
        },
    );
    parent
        .commands()
        .entity(id)
        .with_children(|button| text(button, label, 16., INK));
}

fn hits(card: Player) -> String {
    if card.threatened_hits.is_empty() {
        "No threatened hits".into()
    } else {
        format!("Threats  {}", card.threatened_hits.join(" / "))
    }
}

fn price(offer: &Offer, balance: u8) -> String {
    let cost = offer.cost;
    match offer.balance_after {
        Some(left) => format!("${cost}  /  ${left} left"),
        None => format!("${cost}  /  ${} short", cost - balance),
    }
}

fn offer_tile(
    parent: &mut ChildSpawnerCommands,
    offer: &Offer,
    balance: u8,
    prints: &Prints,
    selection: Option<Selection>,
) {
    let slot = Selection::Offer(offer.slot);
    let id = button(
        parent,
        SeatName::Component(format!("inspect-slot-{}", offer.slot)),
        format!("Inspect {}", offer.player.name),
        Input::Inspect(slot),
        if selection == Some(slot) { GOLD } else { CREAM },
        Node {
            flex_grow: 1.,
            flex_basis: px(0.),
            min_width: px(0.),
            padding: UiRect::all(px(9.)),
            ..row(10.)
        },
    );
    parent.commands().entity(id).with_children(|tile| {
        picture(
            tile,
            prints.faces[&offer.player.printed_face].clone(),
            78.,
            108.,
        );
        tile.spawn(col(3.)).with_children(|info| {
            text(info, offer.player.name, 19., INK);
            text(info, offer.player.immediate_action, 14., MUTED);
            text(info, hits(offer.player), 14., MUTED);
            text(info, price(offer, balance), 15., INK);
        });
    });
}

fn in_play(
    parent: &mut ChildSpawnerCommands,
    players: &[Player],
    prints: &Prints,
    selection: Option<Selection>,
) {
    parent.spawn(row(6.)).with_children(|rail| {
        for &player in players {
            let item = Selection::InPlay(player.card);
            let id = button(
                rail,
                SeatName::Component(format!("inspect-{}", player.card.identity)),
                format!("Inspect {}", player.card.identity),
                Input::Inspect(item),
                if selection == Some(item) { GOLD } else { PAPER },
                Node {
                    flex_basis: px(0.),
                    flex_grow: 1.,
                    min_width: px(0.),
                    padding: UiRect::all(px(6.)),
                    ..row(5.)
                },
            );
            rail.commands().entity(id).with_children(|tile| {
                picture(tile, prints.faces[&player.printed_face].clone(), 33., 46.);
                tile.spawn(col(2.)).with_children(|label| {
                    text(label, player.card.identity.to_string(), 14., INK);
                    text(label, player.name, 13., MUTED);
                });
            });
        }
    });
}

fn inspector(
    parent: &mut ChildSpawnerCommands,
    view: &Observation,
    prints: &Prints,
    selection: Option<Selection>,
) {
    parent.spawn((
        Node { width: px(282.), flex_shrink: 0., align_items: AlignItems::Center, padding: UiRect::all(px(12.)), border_radius: BorderRadius::all(px(6.)), ..col(8.) },
        BackgroundColor(PAPER),
    )).with_children(|panel| {
        let (player, action) = match selection {
            Some(Selection::Offer(slot)) => {
                let offer = &view.offers[usize::from(slot - 1)];
                let permitted = matches!(&view.decision, Some(Decision::Sign { affordable }) if affordable.contains(&offer.player.card));
                text(panel, price(offer, view.balance), 16., MUTED);
                (offer.player, permitted.then(|| (format!("Sign {} for ${}", offer.player.name, offer.cost), Action::Sign { slot, card: offer.player.card })))
            }
            Some(Selection::InPlay(card)) => {
                let Some(player) = view.rosters[0].in_play.iter().find(|player| player.card == card) else { return };
                let permitted = matches!(&view.decision, Some(Decision::Demote { eligible }) if eligible.contains(&card));
                ( *player, permitted.then(|| ("Send to the Minor Leagues".into(), Action::Demote { card })) )
            }
            None => {
                text(panel, "Select a player to inspect the full face.", 18., MUTED);
                return;
            }
        };
        text(panel, player.name, 20., INK);
        let pinch = if player.pinch_hitter { " / PH" } else { "" };
        text(panel, format!("{} / {} / ${} revenue{pinch}", player.card.identity, player.speed, player.revenue), 14., MUTED);
        picture(panel, prints.faces[&player.printed_face].clone(), 232., 322.);
        if let Some((label, action)) = action {
            let name = match action {
                Action::Sign { slot, card } => format!("sign-{slot}-{}", card.instance),
                Action::Demote { card } => format!("demote-{}-{}", card.identity, card.instance),
            };
            short_button(panel, SeatName::Choice(name), label, Input::Submit(view.submission(action)), GOLD);
        }
        short_button(panel, SeatName::Component("close-inspection".into()), "Close face".into(), Input::Close, CREAM);
    });
}

fn event_text(event: Event) -> String {
    match event {
        Event::Signed {
            card,
            paid,
            replacement,
            ..
        } => {
            let player = Player::describe(card).expect("a signed player is public");
            let reveal = Player::describe(replacement).expect("a refill is public");
            format!(
                "SF signed {} for ${paid}. {} entered the same slot.",
                player.name, reveal.name
            )
        }
        Event::Demoted { card } => format!("SF sent {} to the Minor Leagues.", card.identity),
    }
}

pub fn draw(
    mut commands: Commands,
    session: Res<Session>,
    ui: Res<Interface>,
    prints: Res<Prints>,
    existing: Query<Entity, With<Board>>,
) {
    if !session.is_changed() && !ui.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let view = session.game.observe(session.seat);
    commands.spawn((
        Board { revision: view.revision },
        Node { width: percent(100.), height: percent(100.), ..col(0.) },
        Pickable::IGNORE,
    )).with_children(|root| {
        root.spawn((
            Node { width: percent(100.), height: px(HEADER), align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween, padding: UiRect::horizontal(px(18.)), ..default() },
            BackgroundColor(DARK),
        )).with_children(|head| {
            text(head, "Expert   /   Game 1 Buy Round", 24., CREAM);
            text(head, format!("{} manager   /   SF balance ${}", view.seat.abbreviation(), view.balance), 20., GOLD);
        });
        root.spawn((Node { height: px(FIELD), flex_shrink: 0., ..default() }, Pickable::IGNORE));
        root.spawn((
            Node { flex_grow: 1., padding: UiRect::axes(px(16.), px(9.)), ..row(12.) },
            BackgroundColor(DARK),
        )).with_children(|body| {
            body.spawn(Node { flex_grow: 1., min_width: px(0.), ..col(8.) }).with_children(|book| {
                text(book, "Buy Row", 23., CREAM);
                for offers in view.offers.chunks(3) {
                    book.spawn(row(8.)).with_children(|tiles| {
                        for offer in offers {
                            offer_tile(tiles, offer, view.balance, &prints, ui.selection);
                        }
                    });
                }
                let sf = &view.rosters[0];
                if view.events.is_empty() {
                    text(book, "SF shops first.", 17., GOLD);
                }
                for &event in &view.events {
                    text(book, event_text(event), 17., GOLD);
                }
                text(book, format!("Free Agent Deck  {}    /    SF Lineup  {}    /    SF Minor Leagues  {}    /    Active roster  {}",
                    view.free_agent_deck, sf.lineup, sf.minor_leagues, sf.active), 15., FIELD_LABEL);
                if matches!(view.events.as_slice(), [Event::Signed { .. }]) {
                    let title = if view.seat == view.buyer { "In Play / Choose one player to demote" } else { "SF In Play / Demotion pending" };
                    text(book, title, 19., CREAM);
                    in_play(book, &sf.in_play, &prints, ui.selection);
                }
                if let Some(error) = &ui.error {
                    text(book, error, 17., GOLD);
                }
            });
            inspector(body, &view, &prints, ui.selection);
        });
    });
}

pub struct AgentTable;

fn controls(world: &mut World) -> Vec<(Entity, Control)> {
    world
        .query::<(Entity, &Control)>()
        .iter(world)
        .map(|(e, control)| (e, control.clone()))
        .collect()
}

fn viewport(world: &mut World) -> Option<Vec2> {
    world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .single(world)
        .ok()
        .map(|window| Vec2::new(window.width(), window.height()))
}

impl seat::Game for AgentTable {
    fn observe(&self, world: &mut World) -> Observed {
        let (view, refresh_error) = {
            let mut session = world.resource_mut::<Session>();
            let refreshed = session.bypass_change_detection().game.refresh();
            if matches!(refreshed, Ok(true)) {
                session.set_changed();
            }
            (session.game.observe(session.seat), refreshed.is_err())
        };
        let observation = serde_json::to_value(&view).expect("public observation serializes");
        let viewport = viewport(world);
        let controls: Vec<_> = controls(world).into_iter().filter_map(|(entity, control)| {
            let target = seat::ui_target(world, entity, viewport)?;
            let (component, choice) = match control.name {
                SeatName::Component(name) => (Some(name), None),
                SeatName::Choice(name) => (None, Some(name)),
            };
            Some(json!({"component": component, "choice": choice, "label": control.label,
                "position": [target.position.x, target.position.y], "on_screen": target.on_screen }))
        }).collect();
        Observed {
            observation,
            table: json!({ "controls": controls, "refresh_error": refresh_error }),
        }
    }

    fn revision(&self, world: &mut World) -> u64 {
        let session = world.resource::<Session>();
        session.game.revision()
    }

    fn resolve(&self, world: &mut World, name: &SeatName) -> Result<Vec<Vec2>, String> {
        let viewport = viewport(world);
        let entity = controls(world)
            .into_iter()
            .find(|(_, control)| control.name == *name)
            .ok_or("The named control is unavailable.")?
            .0;
        let target = seat::ui_target(world, entity, viewport)
            .filter(|target| target.on_screen)
            .ok_or("The control is outside the table view.")?;
        Ok(vec![target.position])
    }

    fn identify(&self, world: &mut World, entity: Entity) -> Hit {
        let mut node = Some(entity);
        while let Some(entity) = node {
            if let Some(control) = world.get::<Control>(entity) {
                return Hit {
                    name: Some(control.name.clone()),
                    description: json!({ "label": control.label }),
                };
            }
            node = world.get::<ChildOf>(entity).map(ChildOf::parent);
        }
        Hit {
            name: None,
            description: json!({ "surface": "Buy Round" }),
        }
    }

    fn unsettled(&self, world: &mut World) -> Vec<String> {
        let revision = self.revision(world);
        let layout = world
            .query::<&Board>()
            .iter(world)
            .any(|board| board.revision == revision);
        if !layout || viewport(world).is_none() {
            return vec!["Buy Row layout".into()];
        }
        Vec::new()
    }

    fn admission(&self, world: &World) -> Admission {
        if world.contains_resource::<Prints>() {
            Admission::Admitted
        } else {
            Admission::Checking
        }
    }
}
