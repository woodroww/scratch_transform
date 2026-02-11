use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use scratch_transform::gizmo::{GizmoPickSource, PickSelection, TransformGizmoPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            TransformGizmoPlugin,
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
            PanOrbitCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)).with_scale(Vec3::splat(5.0)),
        PickSelection { is_selected: false },
    ));

    let tan = Color::srgb_u8(204, 178, 153);
    let red = Color::srgb_u8(127, 26, 26);

    // cube
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.0)))),
            MeshMaterial3d(materials.add(StandardMaterial::from(red))),
            Transform::from_xyz(-1.0, 0.0, 0.0),
            PickSelection { is_selected: false },
            Visibility::Hidden,
        ))
        .observe(cube_click)
        .with_children(|commands| {
            commands
                .spawn((
                    Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.0)))),
                    MeshMaterial3d(materials.add(StandardMaterial::from(tan))),
                    Transform::from_xyz(1.0, 0.0, 0.0),
                    PickSelection { is_selected: false },
                ))
                .observe(cube_click);
            commands
                .spawn((
                    Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.0)))),
                    MeshMaterial3d(materials.add(StandardMaterial::from(tan))),
                    Transform::from_xyz(1.0, 1.0, 0.0),
                    PickSelection { is_selected: false },
                ))
                .observe(cube_click);
        });

    // light
    commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            button_orbit: MouseButton::Middle,
            ..default()
        },
        GizmoPickSource,
    ));
}

fn cube_click(mut click: On<Pointer<Click>>, mut cubes: Query<(Entity, &mut PickSelection)>) {
    click.propagate(false);
    for (_, mut cube) in cubes.iter_mut() {
        cube.is_selected = false;
    }
    match cubes.get_mut(click.entity) {
        Ok((_, mut cube)) => {
            info!("cube clicked");
            cube.is_selected = true;
        }
        Err(_) => {
            warn!("cube_click error");
        }
    }
}
