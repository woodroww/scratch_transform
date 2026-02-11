use bevy::prelude::*;

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
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let length = 10.0;
    let width = 0.01;
    let gray = Color::srgb(0.2, 0.2, 0.2);

    let x = Cuboid::new(length, width, width);
    let red = Color::srgb(1.0, 0.0, 0.0);

    let y = Cuboid::new(width, length, width);
    let green = Color::srgb(0.0, 1.0, 0.0);

    let z = Cuboid::new(width, width, length);
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
        let x_translation = Vec3::new(length / 2.0, 0.0, 0.0);
        parent.spawn((
            Name::from("x-axis red"),
            Mesh3d(meshes.add(x)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: red,
                ..Default::default()
            })),
            Transform::from_translation(x_translation),
        ));
        parent.spawn((
            Name::from("negative x-axis gray"),
            Mesh3d(meshes.add(x)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: gray,
                ..Default::default()
            })),
            Transform::from_translation(x_translation)
                .rotate_y(180_f32.to_radians()),
        ));

        let y_translation = Vec3::new(0.0, length / 2.0, 0.0);
        parent.spawn((
            Name::from("y-axis green"),
            Mesh3d(meshes.add(y)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: green,
                ..Default::default()
            })),
            Transform::from_translation(y_translation),
        ));
        parent.spawn((
            Name::from("negative y-axis green"),
            Mesh3d(meshes.add(y)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: gray,
                ..Default::default()
            })),
            Transform::from_translation(y_translation)
                .rotate_x(180_f32.to_radians()),
        ));

        let z_translation = Vec3::new(0.0, 0.0, length / 2.0);
        parent.spawn((
            Name::from("z-axis blue"),
            Mesh3d(meshes.add(z)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: blue,
                ..Default::default()
            })),
            Transform::from_translation(z_translation)
        ));
        parent.spawn((
            Name::from("negative z-axis gray"),
            Mesh3d(meshes.add(z)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: gray,
                ..Default::default()
            })),
            Transform::from_translation(z_translation)
                .rotate_z(180_f32.to_radians()),
        ));
    });
}
