use super::{Prints, Session, between_games, card_focus, field, market, market_transfer, revenue};
use accesskit::{Action as AccessibleAction, Node as AccessibleNode, Role};
use baseball_highlights_unfolded::{
    Action, Card, Decision, Event, Observation, Player, Refusal, Seat, Submission,
};
use bevy::{
    a11y::{AccessibilityNode, ActionRequest},
    input_focus::{FocusCause, InputFocus},
    picking::{events::PointerClick, hover::Hovered, pointer::PointerButton, prelude::Pickable},
    prelude::*,
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
    History,
    Glove(card_focus::Target),
}

#[derive(Resource, Default)]
pub struct Interface {
    pub(super) selection: Option<Selection>,
    pub(super) error: Option<String>,
}

impl Interface {
    pub(super) fn inspect(&mut self, selection: Selection) {
        self.selection = Some(selection);
        self.error = None;
    }
}

#[derive(Clone)]
pub(super) enum Input {
    Inspect(Selection),
    ReadCard(card_focus::Region),
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
    controls: Query<(&Control, Option<&card_focus::Region>)>,
    mut session: ResMut<Session>,
    mut ui: ResMut<Interface>,
    mut card_focus: ResMut<card_focus::Interaction>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Ok((control, region)) = controls.get(event.entity) else {
        return;
    };
    activate(control, region, &mut session, &mut ui, &mut card_focus);
}

pub(super) fn accessibility(
    mut requests: MessageReader<ActionRequest>,
    controls: Query<(&Control, Option<&card_focus::Region>)>,
    mut session: ResMut<Session>,
    mut ui: ResMut<Interface>,
    mut card_focus: ResMut<card_focus::Interaction>,
) {
    for request in requests.read() {
        if request.action == AccessibleAction::Click
            && let Some(entity) = Entity::try_from_bits(request.target_node.0)
            && let Ok((control, region)) = controls.get(entity)
        {
            activate(control, region, &mut session, &mut ui, &mut card_focus);
            if let Some(region) = region {
                card_focus.focus_accessible(*region);
            }
        }
    }
}

pub(super) fn accessibility_focus(
    card_focus: Res<card_focus::Interaction>,
    controls: Query<(Entity, &card_focus::Region), With<Control>>,
    mut focus: ResMut<InputFocus>,
) {
    let next = card_focus.keyboard_focus().and_then(|active| {
        controls
            .iter()
            .find_map(|(entity, region)| (active == *region).then_some(entity))
    });
    if focus.get() != next {
        if let Some(entity) = next {
            focus.set(entity, FocusCause::Navigated);
        } else {
            // Bevy still routes keys to the window with no focused control.
            focus.clear();
        }
    }
}

fn activate(
    control: &Control,
    region: Option<&card_focus::Region>,
    session: &mut ResMut<Session>,
    ui: &mut ResMut<Interface>,
    card_focus: &mut ResMut<card_focus::Interaction>,
) {
    match &control.input {
        Input::Inspect(selection) => {
            ui.inspect(*selection);
            if let Some(region) = region {
                card_focus.click(*region);
            }
        }
        Input::ReadCard(region) => card_focus.click(*region),
        Input::Close => {
            ui.selection = None;
            ui.error = None;
        }
        Input::Submit(submission) => submit(session, ui, submission.clone()),
    }
}

pub(super) fn submit(session: &mut Session, ui: &mut Interface, submission: Submission) {
    if session.seat != submission.seat {
        ui.error = Some("This decision belongs to the other manager.".into());
        return;
    }
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
        (Changed<Hovered>, Without<card_focus::Region>),
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
    let mut accessible = AccessibleNode::new(Role::Button);
    accessible.set_label(label.as_str());
    accessible.add_action(AccessibleAction::Click);
    parent
        .spawn((
            Pickable::default(),
            Node {
                border_radius: BorderRadius::all(px(5.)),
                ..node
            },
            BackgroundColor(color),
            Hovered::default(),
            AccessibilityNode::from(accessible),
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

fn refresh_session(session: &mut Mut<Session>) -> Result<bool, Refusal> {
    let advanced = session.bypass_change_detection().game.refresh()?;
    if advanced {
        session.set_changed();
    }
    Ok(advanced)
}

pub fn refresh(
    time: Res<Time>,
    mut elapsed: Local<std::time::Duration>,
    session: ResMut<Session>,
    mut ui: ResMut<Interface>,
) {
    *elapsed += time.delta();
    if *elapsed < std::time::Duration::from_millis(250) {
        return;
    }
    *elapsed = std::time::Duration::ZERO;
    if matches!(refresh_session(&mut session.into()), Ok(true)) {
        *ui = Interface::default();
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
            field::Layout::new(size),
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
        let layout = market::Layout::new(size, view.demotion_pending);
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
        let (view, refreshed) = {
            let mut session = world.resource_mut::<Session>();
            let refreshed = refresh_session(&mut session);
            (session.game.observe(session.seat), refreshed)
        };
        if matches!(refreshed, Ok(true)) {
            *world.resource_mut::<Interface>() = Interface::default();
        }
        let refresh_error = refreshed.is_err();
        let selection = world.resource::<Interface>().selection;
        let revenue_inspection = revenue::inspected_contributor(&view, selection);
        let dugout_inspection = between_games::inspection(&view, selection);
        let round_history =
            (selection == Some(Selection::History) && view.field.is_none()).then(|| {
                view.events
                    .iter()
                    .copied()
                    .map(event_text)
                    .collect::<Vec<_>>()
            });
        let interaction = world.resource::<card_focus::Interaction>();
        let market_inspection = market::reading(&view, interaction.visible());
        let hand_inspection = field::hand::reading(&view, interaction.visible());
        let glove_inspection = field::glove::reading(&view, interaction.visible());
        let focus = interaction.focus();
        let cards_in_motion = world.resource::<market_transfer::Playback>().moving();
        let observation = serde_json::to_value(&view).expect("public observation serializes");
        let viewport = viewport(world);
        let stadium = view.field.as_ref().zip(viewport).map(|(game, size)| {
            let rect = field::Layout::new(size).stadium();
            json!({
                "viewport": [rect.min.x, rect.min.y, rect.width(), rect.height()],
                "prompt": field::prompt(&view, game),
                "threat": field::threat_summary(game),
                "recent_plays": field::recent_plays(&view),
            })
        });
        let mut focused = None;
        let controls: Vec<_> = controls(world).into_iter().filter_map(|(entity, control)| {
            let target = seat::ui_target(world, entity, viewport)?;
            let (component, choice) = match control.name {
                SeatName::Component(name) => (Some(name), None),
                SeatName::Choice(name) => (None, Some(name)),
            };
            if focus.is_some() && world.get::<card_focus::Region>(entity).copied() == focus {
                focused = Some(json!({ "component": component, "choice": choice, "label": control.label }));
            }
            Some(json!({"component": component, "choice": choice, "label": control.label,
                "position": [target.position.x, target.position.y], "on_screen": target.on_screen }))
        }).collect();
        Observed {
            observation,
            table: json!({
                "controls": controls, "focus": focused,
                "supported_keys": card_focus::KEYS.iter().map(|key| key.name).collect::<Vec<_>>(),
                "market_inspection": market_inspection, "cards_in_motion": cards_in_motion,
                "stadium": stadium, "hand_inspection": hand_inspection,
                "glove_inspection": glove_inspection,
                "round_history": round_history,
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
        if world.resource::<card_focus::Interaction>().waiting() {
            return vec!["Card help".into()];
        }
        if world.resource::<market_transfer::Playback>().moving() {
            return vec!["Card transfer".into()];
        }
        Vec::new()
    }

    fn focus(&self, world: &mut World) -> seat::Focus {
        let Some(focus) = world.resource::<card_focus::Interaction>().focus() else {
            return seat::Focus::Nothing;
        };
        for (entity, control) in controls(world) {
            if world.get::<card_focus::Region>(entity).copied() == Some(focus) {
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
        &card_focus::KEYS
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
