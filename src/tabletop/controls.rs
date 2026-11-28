use super::{Prints, Session, between_games, field, market, revenue};
use baseball_highlights_unfolded::{
    Action, Card, Decision, Event, HitBox, Observation, Player, Seat, Submission,
};
use bevy::{
    picking::{events::PointerClick, hover::Hovered, pointer::PointerButton, prelude::Pickable},
    prelude::*,
    ui::prelude::AccessibleLabel,
    window::PrimaryWindow,
};
use serde_json::json;
use unfolded_tabletop::seat::{self, Admission, Hit, Name as SeatName, Observed};

pub(super) const INK: Color = Color::srgb(0.14, 0.10, 0.07);
pub(super) const MUTED: Color = Color::srgb(0.34, 0.24, 0.15);
pub(super) const CREAM: Color = Color::srgb(0.97, 0.91, 0.80);
pub(super) const PAPER: Color = Color::srgb(0.92, 0.84, 0.69);
pub(super) const GOLD: Color = Color::srgb(0.96, 0.78, 0.48);
pub(super) const DARK: Color = Color::srgb(0.10, 0.16, 0.13);
pub(super) const FIELD_LABEL: Color = Color::srgb(0.81, 0.88, 0.77);

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Selection {
    Offer { slot: u8, card: Card },
    PublicCard(Card),
    Dugout(Seat),
    DugoutCard { seat: Seat, card: Card },
    HandCard(Card),
    History,
    Glove(HitBox),
}

#[derive(Resource, Default)]
pub struct Interface {
    pub(super) selection: Option<Selection>,
    pub(super) error: Option<String>,
}

#[derive(Clone)]
pub(super) enum Input {
    Inspect(Selection),
    ReadOffer(market::Region),
    Submit(Submission),
    Close,
}

#[derive(Component, Clone)]
pub(super) struct Control {
    name: SeatName,
    label: String,
    pub(super) input: Input,
    color: Color,
}

#[derive(Component, Clone)]
pub struct Board {
    pub(super) revision: u64,
}

fn click(
    event: On<PointerClick>,
    controls: Query<&Control>,
    mut session: ResMut<Session>,
    mut ui: ResMut<Interface>,
    mut market: ResMut<market::Interaction>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    match &control.input {
        Input::Inspect(selection) => {
            ui.selection = Some(*selection);
            ui.error = None;
        }
        Input::ReadOffer(region) => market.click(*region),
        Input::Close => {
            ui.selection = None;
            ui.error = None;
        }
        Input::Submit(submission) => submit(&mut session, &mut ui, submission.clone()),
    }
}

pub(super) fn submit(session: &mut Session, ui: &mut Interface, submission: Submission) {
    match session.game.submit(submission) {
        Ok(()) => {
            ui.selection = None;
            ui.error = None;
        }
        Err(error) => ui.error = Some(error.to_string()),
    }
}

pub fn hover(
    mut buttons: Query<
        (
            &Hovered,
            &Control,
            &mut BackgroundColor,
            Has<revenue::ReadingControl>,
        ),
        (Changed<Hovered>, Without<market::Region>),
    >,
) {
    for (hovered, control, mut color, reading) in &mut buttons {
        color.0 = match (hovered.get(), reading) {
            (true, true) => revenue::READ_HIGHLIGHT,
            (true, false) => GOLD,
            (false, _) => control.color,
        };
    }
}

pub(super) fn col(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(gap),
        ..default()
    }
}

pub(super) fn row(gap: f32) -> Node {
    Node {
        column_gap: px(gap),
        ..default()
    }
}

pub(super) fn text(
    parent: &mut ChildSpawnerCommands,
    words: impl Into<String>,
    size: f32,
    color: Color,
) {
    parent.spawn((
        Text::new(words),
        Label,
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    ));
}

pub(super) fn picture(
    parent: &mut ChildSpawnerCommands,
    handle: Handle<Image>,
    width: f32,
    height: f32,
) {
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

pub(super) fn button(
    parent: &mut ChildSpawnerCommands,
    name: SeatName,
    label: impl Into<String>,
    input: Input,
    color: Color,
    node: Node,
) -> Entity {
    let label = label.into();
    parent
        .spawn((
            Pickable::default(),
            Node {
                border_radius: BorderRadius::all(px(5.)),
                ..node
            },
            BackgroundColor(color),
            Hovered::default(),
            AccessibleLabel::new(label.clone()),
            Control {
                name,
                label,
                input,
                color,
            },
        ))
        .observe(click)
        .id()
}

pub(super) fn short_button(
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

pub(super) fn hits(card: Player) -> String {
    if card.threatened_hits.is_empty() {
        "No threatened hits".into()
    } else {
        format!("Threats  {}", card.threatened_hits.join(" / "))
    }
}

pub(super) fn card_rail(
    parent: &mut ChildSpawnerCommands,
    players: &[Player],
    prints: &Prints,
    selection: Option<Selection>,
    selection_for: fn(Card) -> Selection,
    first_position: usize,
) {
    parent.spawn(row(6.)).with_children(|rail| {
        for (position, &player) in players.iter().enumerate() {
            let item = selection_for(player.card);
            let id = button(
                rail,
                SeatName::Component(format!("inspect-{}", player.card.identity)),
                format!(
                    "Inspect {}, position {}",
                    player.name,
                    first_position + position
                ),
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
                picture(tile, prints.front(player.card.identity), 28., 39.);
                text(tile, player.name, 14., INK);
            });
        }
    });
}

pub(super) fn inspector(
    parent: &mut ChildSpawnerCommands,
    view: &Observation,
    prints: &Prints,
    selection: Option<Selection>,
    scale: f32,
) {
    let panel = parent
        .spawn((
            Node {
                width: percent(100.),
                height: percent(100.),
                padding: UiRect::all(px(12. * scale)),
                border_radius: BorderRadius::all(px(6. * scale)),
                ..col(7. * scale)
            },
            BackgroundColor(PAPER),
        ))
        .id();
    parent.commands().entity(panel).with_children(|panel| {
        if selection == Some(Selection::History) {
            text(panel, "Buy Round history", 20. * scale, INK);
            panel.spawn(Node {
                flex_grow: 1.,
                min_height: px(0.),
                overflow: Overflow::clip_y(),
                ..col(5. * scale)
            }).with_children(|history| {
                for &event in &view.events {
                    text(history, event_text(event), 14. * scale, INK);
                }
            });
            short_button(panel, SeatName::Component("close-history".into()), "Close history".into(), Input::Close, CREAM);
            return;
        }
        let Some(Selection::PublicCard(card)) = selection else {
            return;
        };
        let Some(player) = view.rosters.iter()
            .flat_map(|roster| roster.in_play.iter().chain(&roster.dugout))
            .find(|player| player.card == card) else {
                text(panel, "This player is no longer in a public pile.", 17. * scale, INK);
                return;
            };
        panel.spawn(row(12. * scale)).with_children(|reading| {
            picture(reading, prints.front(player.card.identity), 139. * scale, 194. * scale);
            reading.spawn(col(8. * scale)).with_children(|details| {
                text(details, player.name, 19. * scale, INK);
                let pinch = if player.pinch_hitter { " / PH" } else { "" };
                text(details, format!("{} / ${} revenue{pinch}", player.speed, player.revenue), 14. * scale, MUTED);
                if matches!(&view.decision, Some(Decision::Demote { eligible }) if eligible.contains(&card)) {
                    short_button(
                        details,
                        SeatName::Choice(format!("demote-{}-{}", card.identity, card.instance)),
                        format!("Send {} to Minors", player.name),
                        Input::Submit(view.submission(Action::Demote { card })),
                        GOLD,
                    );
                }
                short_button(details, SeatName::Component("close-inspection".into()), "Close face".into(), Input::Close, CREAM);
            });
        });
    });
}

pub(super) fn event_text(event: Event) -> String {
    match event {
        Event::OrderChosen { chooser, first } => format!(
            "{} chose {} to buy first.",
            chooser.abbreviation(),
            first.abbreviation(),
        ),
        Event::Signed {
            seat,
            card,
            paid,
            replacement,
            ..
        } => {
            let player = Player::describe(card).expect("a signed player is public");
            let reveal = Player::describe(replacement).expect("a refill is public");
            format!(
                "{} signed {} for ${paid}. {} entered the same slot.",
                seat.abbreviation(),
                player.name,
                reveal.name,
            )
        }
        Event::Demoted { seat, card } => {
            let player = Player::describe(card).expect("a demoted player was public");
            format!(
                "{} sent {} (${} revenue) to the Minor Leagues.",
                seat.abbreviation(),
                player.name,
                player.revenue,
            )
        }
        Event::Passed { seat, unspent } => format!(
            "{} finished buying. ${unspent} expired.",
            seat.abbreviation(),
        ),
        _ => unreachable!("only Buy Round events reach the Buy Round panel"),
    }
}

pub fn refresh(
    time: Res<Time>,
    mut elapsed: Local<std::time::Duration>,
    mut session: ResMut<Session>,
) {
    *elapsed += time.delta();
    if *elapsed < std::time::Duration::from_millis(250) {
        return;
    }
    *elapsed = std::time::Duration::ZERO;
    if matches!(session.bypass_change_detection().game.refresh(), Ok(true)) {
        session.set_changed();
    }
}

pub fn draw(
    mut commands: Commands,
    session: Res<Session>,
    ui: Res<Interface>,
    prints: Res<Prints>,
    existing: Query<Entity, With<Board>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut last_size: Local<Vec2>,
) {
    let size = Vec2::new(window.width(), window.height());
    if !session.is_changed() && !ui.is_changed() && *last_size == size {
        return;
    }
    *last_size = size;
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let view = session.game.observe(session.seat);
    if let Some(position) = &view.field {
        field::draw(
            &mut commands,
            &view,
            position,
            &prints,
            ui.selection,
            ui.error.as_deref(),
        );
        return;
    }
    let mut board = commands.spawn((
        Board {
            revision: view.revision,
        },
        Node {
            width: percent(100.),
            height: percent(100.),
            ..col(0.)
        },
        Pickable::IGNORE,
    ));
    if let Some(next_game) = view.next_game {
        board.with_children(|root| {
            between_games::draw(
                root,
                &view,
                next_game,
                &prints,
                ui.selection,
                ui.error.as_deref(),
            )
        });
    } else if view.first_buyer.is_none() {
        board.with_children(|root| {
            revenue::draw(root, &view, &prints, ui.selection, ui.error.as_deref())
        });
    } else {
        let layout = market::Layout::new(size);
        board.insert((layout, market::background()));
        board.with_children(|root| market::draw(root, layout, &view, &prints, &ui));
    }
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
        let revenue_inspection =
            revenue::inspected_contributor(&view, world.resource::<Interface>().selection);
        let dugout_inspection =
            between_games::inspection(&view, world.resource::<Interface>().selection);
        let interaction = world.resource::<market::Interaction>();
        let market_inspection = interaction.reading(&view);
        let focus = interaction.focus();
        let observation = serde_json::to_value(&view).expect("public observation serializes");
        let viewport = viewport(world);
        let mut focused = None;
        let controls: Vec<_> = controls(world).into_iter().filter_map(|(entity, control)| {
            let target = seat::ui_target(world, entity, viewport)?;
            let (component, choice) = match control.name {
                SeatName::Component(name) => (Some(name), None),
                SeatName::Choice(name) => (None, Some(name)),
            };
            if focus.is_some() && world.get::<market::Region>(entity).copied() == focus {
                focused = Some(json!({ "component": component, "choice": choice, "label": control.label }));
            }
            Some(json!({"component": component, "choice": choice, "label": control.label,
                "position": [target.position.x, target.position.y], "on_screen": target.on_screen }))
        }).collect();
        Observed {
            observation,
            table: json!({
                "controls": controls, "focus": focused,
                "supported_keys": market::KEYS.iter().map(|key| key.name).collect::<Vec<_>>(),
                "market_inspection": market_inspection,
                "refresh_error": refresh_error, "revenue_inspection": revenue_inspection,
                "dugout_inspection": dugout_inspection,
            }),
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
            description: json!({ "surface": if world.resource::<Session>().game.has_field() {
                "Field"
            } else {
                "Buy Round"
            } }),
        }
    }

    fn unsettled(&self, world: &mut World) -> Vec<String> {
        let revision = self.revision(world);
        let layout = world
            .query::<&Board>()
            .iter(world)
            .any(|board| board.revision == revision);
        if !layout || viewport(world).is_none() {
            return vec!["Table layout".into()];
        }
        if world.resource::<market::Interaction>().waiting() {
            return vec!["Card help".into()];
        }
        Vec::new()
    }

    fn focus(&self, world: &mut World) -> seat::Focus {
        let Some(focus) = world.resource::<market::Interaction>().focus() else {
            return seat::Focus::Nothing;
        };
        for (entity, control) in controls(world) {
            if world.get::<market::Region>(entity).copied() == Some(focus) {
                return seat::Focus::Control {
                    choice: match control.name {
                        SeatName::Choice(name) => Some(name),
                        SeatName::Component(_) => None,
                    },
                    label: control.label,
                };
            }
        }
        seat::Focus::Nothing
    }

    fn keys(&self) -> &[seat::SeatKey] {
        &market::KEYS
    }

    fn interrupts(&self, _world: &mut World, input: seat::Interrupt<'_>) -> bool {
        matches!(
            input,
            seat::Interrupt::Key("Tab" | "Shift+Tab" | "Escape" | "Space")
                | seat::Interrupt::Activate(SeatName::Component(_))
        )
    }

    fn admission(&self, world: &World) -> Admission {
        if world.contains_resource::<Prints>() {
            Admission::Admitted
        } else {
            Admission::Checking
        }
    }
}
