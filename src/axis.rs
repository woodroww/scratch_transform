use bevy::prelude::*;

use crate::gizmo_material::GizmoMaterial;

pub struct AxisPlugin;

impl Plugin for AxisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_main_axis)
            .add_systems(Update, keyboard_axis);
    }
}

fn keyboard_axis(
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<&mut Visibility, With<AxisMarker>>,
) {
    if keyboard.just_pressed(KeyCode::KeyA) {
        for mut vis in query {
            *vis = match *vis {
                Visibility::Inherited => Visibility::Inherited,
                Visibility::Hidden => Visibility::Visible,
                Visibility::Visible => Visibility::Hidden,
            };
        }
    }
}

#[derive(Component)]
struct AxisMarker;

fn spawn_main_axis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GizmoMaterial>>,
) {
    let length = 10.0;
    let width = 0.01;
    let gray = Color::srgb(0.4, 0.4, 0.4);

    let x = Cylinder::new(width, length);
    let red = Color::srgb(1.0, 0.0, 0.0);

    let y = Cylinder::new(width, length);
    let green = Color::srgb(0.0, 1.0, 0.0);

    let z = Cylinder::new(width, length);
    let blue = Color::srgb(0.0, 0.0, 1.0);

    let empty: Entity = commands
        .spawn((
            Transform::from_translation(Vec3::ZERO),
            Visibility::Visible,
            InheritedVisibility::default(),
            Name::from("Main Axis"),
            AxisMarker,
        ))
        .id();

    commands.entity(empty).with_children(|parent| {
        let mut x_transform = Transform::default();//from_translation(x_translation);
        x_transform.rotate_z(90_f32.to_radians());
        x_transform.translation.x += length / 2.0;
        parent.spawn((
            Name::from("x-axis red"),
            Mesh3d(meshes.add(x)),
            MeshMaterial3d(materials.add(GizmoMaterial {
                color: red.into(),
            })),
            x_transform,
        ));

        x_transform.translation.x -= length;
        parent.spawn((
            Name::from("negative x-axis gray"),
            Mesh3d(meshes.add(x)),
            MeshMaterial3d(materials.add(GizmoMaterial {
                color: gray.into(),
            })),
            x_transform,
        ));

        let mut y_transform = Transform::default();//from_translation(x_translation);
        y_transform.rotate_y(90_f32.to_radians());
        y_transform.translation.y += length / 2.0;
        parent.spawn((
            Name::from("y-axis green"),
            Mesh3d(meshes.add(y)),
            MeshMaterial3d(materials.add(GizmoMaterial {
                color: green.into(),
            })),
            y_transform,
        ));

        y_transform.translation.y -= length;
        parent.spawn((
            Name::from("negative y-axis green"),
            Mesh3d(meshes.add(y)),
            MeshMaterial3d(materials.add(GizmoMaterial {
                color: gray.into(),
            })),
            y_transform,
        ));

        let mut z_transform = Transform::default();
        z_transform.rotate_x(90_f32.to_radians());
        z_transform.translation.z += length / 2.0;
        parent.spawn((
            Name::from("z-axis blue"),
            Mesh3d(meshes.add(z)),
            MeshMaterial3d(materials.add(GizmoMaterial {
                color: blue.into(),
            })),
            z_transform,
        ));

        z_transform.translation.z -= length;
        parent.spawn((
            Name::from("negative z-axis gray"),
            Mesh3d(meshes.add(z)),
            MeshMaterial3d(materials.add(GizmoMaterial {
                color: gray.into(),
            })),
            z_transform,
        ));
    });
}
