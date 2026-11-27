//! Textured solids present the committed zones. Transfers are cosmetic playback.
use super::{FIELD, HEADER, Prints, Session};
use baseball_highlights_unfolded::{CardId, Event};
use bevy::{
    camera::{ScalingMode, Viewport},
    prelude::*,
    render::view::NoIndirectDrawing,
    window::PrimaryWindow,
};
use std::{
    collections::HashMap,
    f32::consts::{FRAC_PI_2, PI},
};
use unfolded_tabletop::{
    capture::OnSurface,
    card::{self, CardPivot, TurningCard},
    card_motion::{CardMove, Lift, Turn},
};

const WIDTH: f32 = 1.03;
const LENGTH: f32 = 1.44;
const THICKNESS: f32 = 0.013;
const DECK: f32 = 3.2;
const LINEUP: f32 = 5.7;
const IN_PLAY: f32 = 8.2;
const MINORS: f32 = 10.7;
const CARD: TurningCard = TurningCard::new(WIDTH, LENGTH, card::CORNER_FRACTION, THICKNESS, 0.001);

#[derive(Component)]
pub struct StageCamera;

#[derive(Resource)]
pub struct Stock {
    paper: Handle<StandardMaterial>,
    back: Handle<StandardMaterial>,
    fronts: HashMap<CardId, Handle<StandardMaterial>>,
    face: Handle<Mesh>,
    solid: Handle<Mesh>,
}

impl Stock {
    fn front(&self, identity: CardId) -> Handle<StandardMaterial> {
        self.fronts[&identity.printed_face()].clone()
    }
}

#[derive(Component)]
pub struct Scene;

#[derive(Component)]
pub struct Motion {
    path: CardMove,
    elapsed: f32,
    stays: bool,
}

fn printed(
    materials: &mut Assets<StandardMaterial>,
    image: Handle<Image>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color_texture: Some(image),
        unlit: true,
        ..default()
    })
}

pub fn setup(
    mut commands: Commands,
    prints: Res<Prints>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        StageCamera,
        Camera3d::default(),
        // The small tray shares meshes and materials across piles. Draw them directly
        // so duplicate face-down piles remain visible on the Vulkan desktop path.
        NoIndirectDrawing,
        OnSurface,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.08, 0.18, 0.12)),
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal {
                viewport_width: 25.5,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0., 13., 6.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(28., 8.))),
        MeshMaterial3d(printed(&mut materials, prints.park.clone())),
        Transform::from_xyz(0., -1.2, 0.).with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 7500.,
            ..default()
        },
        Transform::from_xyz(1., 4., 3.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(Stock {
        paper: materials.add(Color::srgb(0.68, 0.50, 0.30)),
        back: printed(&mut materials, prints.back.clone()),
        fronts: prints
            .faces
            .iter()
            .map(|(&id, image)| (id, printed(&mut materials, image.clone())))
            .collect(),
        face: meshes.add(CARD.face_mesh()),
        solid: meshes.add(CARD.paper_mesh()),
    });
}

pub fn viewport(
    window: Single<&Window, With<PrimaryWindow>>,
    mut camera: Single<&mut Camera, With<StageCamera>>,
) {
    let scale = window.scale_factor();
    let viewport = Viewport {
        physical_position: UVec2::new(0, (HEADER * scale).round() as u32),
        physical_size: UVec2::new(window.physical_width(), (FIELD * scale).round() as u32),
        ..default()
    };
    if !camera.viewport.as_ref().is_some_and(|current| {
        current.physical_position == viewport.physical_position
            && current.physical_size == viewport.physical_size
    }) {
        camera.viewport = Some(viewport);
    }
}

fn position(x: f32) -> Vec3 {
    Vec3::new(x, 0., 0.)
}

fn slot_at(slot: u8) -> Vec3 {
    position(-10.5 + (f32::from(slot) - 1.) * 2.05)
}

fn pile_top_at(x: f32, count: usize) -> Vec3 {
    position(x) + Vec3::Y * (THICKNESS * count as f32)
}

fn card_model(
    commands: &mut Commands,
    stock: &Stock,
    at: Transform,
    front: Handle<StandardMaterial>,
) -> Entity {
    let poses = CARD.poses(CardPivot::Center);
    commands
        .spawn((Scene, at, Visibility::default()))
        .with_children(|card| {
            card.spawn((
                Mesh3d(stock.solid.clone()),
                MeshMaterial3d(stock.paper.clone()),
                poses.paper,
            ));
            card.spawn((
                Mesh3d(stock.face.clone()),
                MeshMaterial3d(front),
                poses.front,
            ));
            card.spawn((
                Mesh3d(stock.face.clone()),
                MeshMaterial3d(stock.back.clone()),
                poses.back,
            ));
        })
        .id()
}

fn pile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    stock: &Stock,
    x: f32,
    count: usize,
    top: Handle<StandardMaterial>,
) {
    if count == 0 {
        return;
    }
    let height = THICKNESS * count as f32;
    commands
        .spawn((
            Scene,
            Transform::from_translation(position(x)),
            Visibility::default(),
        ))
        .with_children(|pile| {
            pile.spawn((
                Mesh3d(meshes.add(card::pile(WIDTH, LENGTH, THICKNESS, count))),
                MeshMaterial3d(stock.paper.clone()),
                Transform::from_xyz(0., height / 2., 0.),
            ));
            pile.spawn((
                Mesh3d(stock.face.clone()),
                MeshMaterial3d(top),
                Transform::from_xyz(0., height + 0.002, 0.),
            ));
        });
}

fn transfer(path: CardMove, face_down: bool) -> CardMove {
    let turn = if face_down {
        Turn {
            axis: Vec3::Z,
            from: 0.,
            to: PI,
            begins: 0.25,
            lasts: 0.75,
        }
    } else {
        Turn::over(Vec3::Z)
    };
    path.lifted(Lift::arc(0.72)).turned(turn)
}

pub fn draw(
    mut commands: Commands,
    session: Res<Session>,
    stock: Res<Stock>,
    mut meshes: ResMut<Assets<Mesh>>,
    existing: Query<Entity, With<Scene>>,
    mut shown: Local<Option<u64>>,
) {
    if *shown == Some(session.game.revision()) {
        return;
    }
    let view = session.game.observe(session.seat);
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let latest = shown.is_some().then(|| view.events.last()).flatten();
    *shown = Some(view.revision);
    let roster = &view.rosters[0];
    for offer in &view.offers {
        let rest = Transform::from_translation(slot_at(offer.slot));
        let moving = matches!(latest, Some(Event::Signed { slot, .. }) if *slot == offer.slot)
            .then(|| {
                transfer(
                    CardMove::new(pile_top_at(DECK, view.free_agent_deck + 1), rest, 0.58),
                    false,
                )
            });
        let active = moving.map_or(rest, |path| path.pose(0.));
        let entity = card_model(
            &mut commands,
            &stock,
            active,
            stock.front(offer.player.printed_face),
        );
        if let Some(path) = moving {
            commands.entity(entity).insert(Motion {
                path,
                elapsed: 0.,
                stays: true,
            });
        }
    }
    let back = stock.back.clone();
    pile(
        &mut commands,
        &mut meshes,
        &stock,
        DECK,
        view.free_agent_deck,
        back.clone(),
    );
    pile(
        &mut commands,
        &mut meshes,
        &stock,
        LINEUP,
        roster.lineup,
        back.clone(),
    );
    pile(
        &mut commands,
        &mut meshes,
        &stock,
        MINORS,
        roster.minor_leagues,
        back,
    );
    if let Some(top) = roster.in_play.last() {
        pile(
            &mut commands,
            &mut meshes,
            &stock,
            IN_PLAY,
            roster.in_play.len(),
            stock.front(top.printed_face),
        );
    }
    let source = match latest {
        Some(Event::Signed { card, slot, .. }) => {
            Some((*card, slot_at(*slot), pile_top_at(LINEUP, roster.lineup)))
        }
        Some(Event::Demoted { card }) => Some((
            *card,
            pile_top_at(IN_PLAY, roster.in_play.len() + 1),
            pile_top_at(MINORS, roster.minor_leagues),
        )),
        None => None,
    };
    if let Some((card, start, end)) = source {
        let path = transfer(
            CardMove::new(start, Transform::from_translation(end), 0.50),
            true,
        );
        let entity = card_model(
            &mut commands,
            &stock,
            path.pose(0.),
            stock.front(card.identity),
        );
        commands.entity(entity).insert(Motion {
            path,
            elapsed: 0.,
            stays: false,
        });
    }
}

pub fn animate(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut moving: Query<(Entity, &mut Motion, &mut Transform)>,
) {
    for (entity, mut motion, mut pose) in &mut moving {
        if keys.just_pressed(KeyCode::Space) {
            motion.elapsed = motion.path.duration;
        } else {
            motion.elapsed += time.delta_secs();
        }
        *pose = motion.path.pose(motion.elapsed);
        if motion.path.finished(motion.elapsed) {
            if motion.stays {
                commands.entity(entity).remove::<Motion>();
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}
