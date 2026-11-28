//! Joint pointer and keyboard focus for an actionable face and its disclosure.
use super::{
    Session,
    controls::{self, Input, Interface, Selection},
    field, market,
};
use baseball_highlights_unfolded::{Card, HitBox, Observation, Seat};
use bevy::{
    input::keyboard::{Key, KeyboardFocusLost, KeyboardInput},
    picking::{hover::Hovered, prelude::Pickable},
    prelude::*,
    window::PrimaryWindow,
};
use std::time::Duration;
use unfolded_hover::Hover;
use unfolded_tabletop::seat;

const DWELL: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Source {
    Hand(u8, Card),
    InPlay(u8, Card),
    GloveFace(u8, Card),
    GloveBox(HitBox),
    UseGlove,
    Offer(u8, Card),
    Finish,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Target {
    revision: u64,
    seat: Seat,
    pub(super) source: Source,
}

impl Target {
    pub(super) fn new(view: &Observation, source: Source) -> Self {
        Self {
            revision: view.revision,
            seat: view.seat,
            source,
        }
    }

    pub(super) fn matches(self, view: &Observation) -> bool {
        self == Self::new(view, self.source)
    }

    fn order(self) -> (u8, u8) {
        match self.source {
            Source::Hand(index, _) | Source::InPlay(index, _) | Source::GloveFace(index, _) => {
                (0, index)
            }
            Source::Offer(slot, _) => (1, slot),
            Source::GloveBox(hit_box) => (1, hit_box.number),
            Source::Finish | Source::UseGlove => (2, 0),
        }
    }

    fn has_help(self) -> bool {
        !matches!(
            self.source,
            Source::Finish | Source::GloveBox(_) | Source::UseGlove
        )
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
    pub(super) part: Part,
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

    pub(super) fn keyboard_focus(&self) -> Option<Region> {
        self.focus.filter(|_| self.keyboard)
    }

    pub(super) fn visible(&self) -> Option<Target> {
        self.hover.visible().copied()
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
    layout: Query<(), Or<(With<market::Layout>, With<field::Layout>)>>,
    regions: Query<(&Region, &Hovered, Option<&controls::Control>)>,
) {
    let mut session = session;
    let active =
        layout.single().is_ok() && matches!(ui.selection, None | Some(Selection::Glove(_)));
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
                && region.target.has_help()
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
                    && focus.target.has_help()
                {
                    focus.part = Part::Face;
                }
            }
            KeyCode::Enter | KeyCode::NumpadEnter => {
                interaction.keyboard = true;
                let focused_input = regions.iter().find_map(|(region, _, control)| {
                    (Some(*region) == interaction.focus)
                        .then(|| control.map(|control| &control.input))
                        .flatten()
                });
                match focused_input {
                    Some(Input::Submit(action)) => {
                        controls::submit(&mut session, &mut ui, action.clone());
                        return;
                    }
                    Some(Input::Inspect(selection @ Selection::Glove(_))) => {
                        ui.inspect(*selection);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    if !active {
        return;
    }
    let target = if interaction.keyboard {
        interaction.focus.filter(|region| region.target.has_help())
    } else {
        clicked
            .filter(|region| region.target.has_help())
            .or(pointed)
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

pub(super) fn bridge(root: &mut ChildSpawnerCommands, target: Target, node: Node) {
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
