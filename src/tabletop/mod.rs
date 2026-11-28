mod controls;
mod field;
mod stage;

use baseball_highlights_unfolded::{CardId, Game, GloveReview, Seat, assets::Collection};
use bevy::{prelude::*, time::TimeUpdateStrategy};
use std::{collections::HashMap, time::Duration};
use unfolded_tabletop::{
    capture,
    seat::start::{self, Start},
};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;
const HEADER: f32 = 56.;
const FIELD: f32 = 132.;

#[derive(Resource)]
struct Session {
    game: Game,
    seat: Seat,
}

#[derive(Resource)]
pub struct Art {
    faces: HashMap<CardId, image::DynamicImage>,
    back: image::DynamicImage,
    buy_park: image::DynamicImage,
    field_park: image::DynamicImage,
}

impl Art {
    pub fn load(collection: &Collection, review: Option<GloveReview>) -> Result<Self, String> {
        let faces = match review {
            Some(fixture) => collection.glove_review_faces(fixture)?,
            None => collection.expert_series_faces()?,
        };
        let park = |bytes| {
            image::load_from_memory(bytes).map_err(|error| format!("Press Box study: {error}"))
        };
        Ok(Self {
            faces,
            back: collection.back()?,
            buy_park: park(include_bytes!("../../art/manager-review/park-0.png"))?,
            field_park: park(include_bytes!("../../art/manager-review/park-field.png"))?,
        })
    }
}

#[derive(Resource, Clone)]
struct Prints {
    faces: HashMap<CardId, Handle<Image>>,
    back: Handle<Image>,
    buy_park: Handle<Image>,
    field_park: Handle<Image>,
}

fn install(app: &mut App, game: Game, seat: Seat, art: Art) {
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let mut texture =
        |image: image::DynamicImage| images.add(Image::from_dynamic(image, true, default()));
    let prints = Prints {
        back: texture(art.back),
        buy_park: texture(art.buy_park),
        field_park: texture(art.field_park),
        faces: art
            .faces
            .into_iter()
            .map(|(id, image)| (id, texture(image)))
            .collect(),
    };
    app.insert_resource(prints)
        .insert_resource(Session { game, seat })
        .init_resource::<controls::Interface>()
        .add_systems(Startup, (camera, stage::setup))
        .add_systems(
            Update,
            (controls::draw, stage::draw, stage::animate, controls::hover).chain(),
        )
        .add_systems(PostUpdate, stage::viewport);
}

fn camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));
}

pub fn run(game: Game, seat: Seat, art: Art) -> Result<(), String> {
    let mut app = App::new();
    let title = if game.glove_review().is_some() {
        "Baseball Highlights / Glove Review"
    } else {
        "Baseball Highlights / Expert Buy Round"
    };
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: title.into(),
            resolution: (WIDTH, HEIGHT).into(),
            resize_constraints: WindowResizeConstraints {
                min_width: 1080.,
                min_height: 720.,
                ..default()
            },
            ..default()
        }),
        ..default()
    }));
    install(&mut app, game, seat, art);
    app.add_systems(Update, controls::refresh.before(controls::draw));
    match app.run() {
        AppExit::Success => Ok(()),
        AppExit::Error(code) => Err(format!("The tabletop closed with status {code}.")),
    }
}

const START: Start = Start::new("BASEBALL_AGENT_SEAT_DETACHED", |_, limit| {
    format!(
        "The Baseball table did not admit the seat within {} seconds.",
        limit.as_secs()
    )
});

pub fn serve(game: Game, seat: Seat, art: Art) -> Result<(), String> {
    START.run(None, || {
        let mut app = App::new();
        app.add_plugins(capture::windowless(DefaultPlugins))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        install(&mut app, game, seat, art);
        capture::install::<Camera2d>(&mut app, UVec2::new(WIDTH, HEIGHT));
        Ok(start::Table::new(app, controls::AgentTable))
    })
}
