//! Textured solids present the committed zones. Transfers are cosmetic playback.
use super::{FIELD, HEADER, Prints, Session, field};
use baseball_highlights_unfolded::{FieldView, RunnerSpeed};
use bevy::{
    camera::{ScalingMode, Viewport},
    prelude::*,
    render::view::NoIndirectDrawing,
    window::PrimaryWindow,
};
use std::f32::consts::FRAC_PI_2;
use unfolded_tabletop::capture::OnSurface;

// Press Box artboard is 720 by 420 points, mapped at 60 points per
// world unit. Base locations are the centres of the authored paths.
const PARK_WIDTH: f32 = 12.;
const PARK_DEPTH: f32 = 7.;
const BASE_CENTRES: [(f32, f32); 3] = [(446.32, 302.6), (360., 236.2), (273.68, 302.6)];

#[derive(Component)]
pub struct StageCamera;

#[derive(Component)]
pub struct Backdrop {
    field: bool,
}

#[derive(Resource)]
pub struct Stock {
    pawn: Handle<Mesh>,
    pawn_colors: [Handle<StandardMaterial>; 3],
    speed_mark: Handle<Mesh>,
    speed_ink: [Handle<StandardMaterial>; 2],
}

#[derive(Component)]
pub struct Scene;

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
    session: Res<Session>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let field = session.game.has_field();
    let (scaling_mode, pose) = camera_pose(field);
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
            scaling_mode,
            ..OrthographicProjection::default_3d()
        }),
        pose,
    ));
    for (is_field, size, y, image) in [
        (false, Vec2::new(28., 8.), -1.2, &prints.buy_park),
        (
            true,
            Vec2::new(PARK_WIDTH, PARK_DEPTH),
            -0.01,
            &prints.field_park,
        ),
    ] {
        commands.spawn((
            Backdrop { field: is_field },
            Mesh3d(meshes.add(Rectangle::from_size(size))),
            MeshMaterial3d(printed(&mut materials, image.clone())),
            Transform::from_xyz(0., y, 0.).with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            backdrop_visibility(field == is_field),
        ));
    }
    commands.spawn((
        DirectionalLight {
            illuminance: 7500.,
            ..default()
        },
        Transform::from_xyz(1., 4., 3.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(Stock {
        pawn: meshes.add(Cylinder::new(0.23, 0.14)),
        pawn_colors: [
            materials.add(Color::srgb(0.96, 0.92, 0.82)),
            materials.add(Color::srgb(0.15, 0.43, 0.72)),
            materials.add(Color::srgb(0.75, 0.19, 0.14)),
        ],
        speed_mark: meshes.add(Cuboid::new(0.033, 0.02, 0.16)),
        speed_ink: [
            materials.add(Color::srgb(0.11, 0.19, 0.19)),
            materials.add(Color::srgb(0.97, 0.91, 0.78)),
        ],
    });
}

fn camera_pose(field: bool) -> (ScalingMode, Transform) {
    if field {
        (
            ScalingMode::FixedVertical {
                viewport_height: PARK_DEPTH,
            },
            Transform::from_xyz(0., 18., 0.).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        )
    } else {
        (
            ScalingMode::FixedHorizontal {
                viewport_width: 25.5,
            },
            Transform::from_xyz(0., 13., 6.).looking_at(Vec3::ZERO, Vec3::Y),
        )
    }
}

fn backdrop_visibility(visible: bool) -> Visibility {
    if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

pub fn viewport(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&mut Camera, &mut Transform, &mut Projection), With<StageCamera>>,
    session: Res<Session>,
    mut backdrops: Query<(&Backdrop, &mut Visibility)>,
) {
    let (mut camera, mut pose, mut projection) = camera.into_inner();
    let scale = window.scale_factor();
    let field = session.game.has_field();
    let viewport = if field {
        let rect = field::Layout::new(Vec2::new(window.width(), window.height())).stadium();
        let start = (rect.min * scale).round().as_uvec2();
        let end = (rect.max * scale).round().as_uvec2();
        Viewport {
            physical_position: start,
            physical_size: end - start,
            ..default()
        }
    } else {
        Viewport {
            physical_position: UVec2::new(0, (HEADER * scale).round() as u32),
            physical_size: UVec2::new(window.physical_width(), (FIELD * scale).round() as u32),
            ..default()
        }
    };
    if !camera.viewport.as_ref().is_some_and(|current| {
        current.physical_position == viewport.physical_position
            && current.physical_size == viewport.physical_size
    }) {
        camera.viewport = Some(viewport);
    }
    let (scaling_mode, next_pose) = camera_pose(field);
    if *pose != next_pose {
        *pose = next_pose;
        if let Projection::Orthographic(projection) = &mut *projection {
            projection.scaling_mode = scaling_mode;
        }
        for (backdrop, mut visibility) in &mut backdrops {
            *visibility = backdrop_visibility(field == backdrop.field);
        }
    }
}

fn field_position(x: f32, y: f32) -> Vec3 {
    Vec3::new((x - 360.) / 60., 0., (y - 210.) / 60.)
}

fn pawn(commands: &mut Commands, stock: &Stock, at: Vec3, speed: RunnerSpeed) {
    let bars = match speed {
        RunnerSpeed::Slow => 1,
        RunnerSpeed::Average => 2,
        RunnerSpeed::Fast => 3,
    };
    commands
        .spawn((
            Scene,
            Mesh3d(stock.pawn.clone()),
            MeshMaterial3d(stock.pawn_colors[bars - 1].clone()),
            Transform::from_translation(at + 0.07 * Vec3::Y),
        ))
        .with_children(|marker| {
            for bar in 0..bars {
                let x = (bar as f32 - (bars - 1) as f32 / 2.) * 0.08;
                marker.spawn((
                    Mesh3d(stock.speed_mark.clone()),
                    MeshMaterial3d(stock.speed_ink[usize::from(bars > 1)].clone()),
                    Transform::from_xyz(x, 0.075, 0.),
                ));
            }
        });
}

fn field_scene(commands: &mut Commands, stock: &Stock, view: &FieldView) {
    for (base, (x, y)) in BASE_CENTRES.into_iter().enumerate() {
        let at = field_position(x, y);
        let [sf, ny] = view.diamonds.map(|diamond| diamond.bases[base]);
        let separation = if sf.is_some() && ny.is_some() {
            0.24
        } else {
            0.
        };
        for (runner, offset) in [(sf, -separation), (ny, separation)] {
            if let Some(runner) = runner {
                pawn(commands, stock, at + offset * Vec3::X, runner.speed);
            }
        }
    }
    let home = field_position(360., 369.);
    for (index, threat) in view.threatened.iter().enumerate() {
        let offset = (index as f32 - (view.threatened.len() - 1) as f32 / 2.) * 0.51;
        pawn(
            commands,
            stock,
            home + Vec3::new(offset, 0., 0.40),
            threat.batter.speed,
        );
    }
}

pub fn draw(
    mut commands: Commands,
    session: Res<Session>,
    stock: Res<Stock>,
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
    *shown = Some(view.revision);
    if let Some(field) = &view.field {
        field_scene(&mut commands, &stock, field);
    }
}
