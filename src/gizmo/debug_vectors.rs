use crate::gizmo::TransformGizmo;
use bevy::{
    color::palettes::css::*, pbr::wireframe::Wireframe,
    prelude::*,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPrimaryContextPass},
    egui::{self, Color32, RichText},
};

use crate::mesh::cone;

pub struct DebugVectorsPlugin;

impl Plugin for DebugVectorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_debug_vectors)
            .insert_resource(DebugVectors {
                vertical_vector: Vec3::default(),
                plane_normal: Vec3::default(),
                ray: Ray3d {
                    origin: Vec3::default(),
                    direction: Dir3::new(Vec3::new(1.0, 1.0, 0.0).normalize()).unwrap(),
                },
                plane_origin: Vec3::default(),
                cursor_plane_intersection: Vec3::default(),
                cursor_vector: Vec3::default(),
                cross_plane: Vec3::default(),
                cross_plane_normal: Vec3::default(),
            })
            //.add_plugins(MaterialPlugin::<GizmoMaterial>::default())
            .add_systems(EguiPrimaryContextPass, debug_ui)
            .add_systems(
                Update,
                update_debug_vectors.run_if(resource_changed::<DebugVectors>),
            );
    }
}

#[derive(Resource)]
pub struct DebugVectors {
    pub vertical_vector: Vec3,
    pub plane_normal: Vec3,
    pub ray: Ray3d,
    pub plane_origin: Vec3,              // initial click
    pub cursor_plane_intersection: Vec3, // drag location
    pub cursor_vector: Vec3,
    pub cross_plane: Vec3,
    pub cross_plane_normal: Vec3,
}

impl DebugVectors {
    fn value(&self, which: WhichDebugVector) -> Vec3 {
        match which {
            WhichDebugVector::VerticalVector => self.vertical_vector,
            WhichDebugVector::PlaneNormal => self.plane_normal,
            WhichDebugVector::PlaneOrigin => self.plane_origin,
            WhichDebugVector::CursorPlaneIntersection => self.cursor_plane_intersection,
            WhichDebugVector::CursorVector => self.cursor_vector,
            WhichDebugVector::PickingRay => *self.ray.direction,
            WhichDebugVector::CrossPlane => self.cross_plane,
            WhichDebugVector::CrossPlaneNormal => self.cross_plane_normal,
        }
    }

    pub const VALUES: [WhichDebugVector; 8] = [
        WhichDebugVector::VerticalVector,
        WhichDebugVector::PlaneNormal,
        WhichDebugVector::PlaneOrigin,
        WhichDebugVector::CursorPlaneIntersection,
        WhichDebugVector::CursorVector,
        WhichDebugVector::PickingRay,
        WhichDebugVector::CrossPlane,
        WhichDebugVector::CrossPlaneNormal,
    ];
}

impl WhichDebugVector {
    pub fn egui_color(&self) -> Color32 {
        let color = self.color().to_srgba();
        Color32::from_rgba_unmultiplied(
            (color.red * 255.0) as u8,
            (color.green * 255.0) as u8,
            (color.blue * 255.0) as u8,
            (color.alpha * 255.0) as u8,
        )
    }
    fn color(&self) -> Color {
        use WhichDebugVector::*;
        match self {
            VerticalVector => AQUA.into(),
            PlaneNormal => RED.into(),
            PlaneOrigin => YELLOW.into(),
            CursorPlaneIntersection => LIME.into(),
            CursorVector => WHITE.into(),
            PickingRay => ORANGE.into(),
            CrossPlane => PURPLE.into(),
            CrossPlaneNormal => PURPLE.into(),
        }
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub enum WhichDebugVector {
    VerticalVector,
    PlaneNormal,
    PlaneOrigin, // initial click // first original plane, plane rotates to stay on plane with 3d drag
    CursorPlaneIntersection, // drag location
    CursorVector,
    PickingRay,
    CrossPlane, // which side determines length sign
    CrossPlaneNormal,
}

pub fn debug_ui(
    mut contexts: EguiContexts,
    vectors: Res<DebugVectors>,
    gizmo_query: Query<&Transform, With<TransformGizmo>>,
) {
    let ctx = contexts.ctx_mut().unwrap();
    egui::Window::new("DebugVectors").show(ctx, |ui| {
        if let Ok(gizmo_transform) = gizmo_query.single() {
            ui.label(format!(
                "gizmo.............{:.3}",
                gizmo_transform.translation
            ));
        } else {
            ui.label("gizmo.............error");
        }
        for item in DebugVectors::VALUES {
            let mut text = RichText::new(format!("{:?}\n{:.2}", item, vectors.value(item)))
                .color(item.egui_color());
            text = text.size(30.0);
            ui.label(text);
        }
    });
}

pub fn spawn_arrow_vector(
    mut commands: Commands,
    vector_type: WhichDebugVector,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let axis_length = 1.0;
    let arrow_tail_mesh = meshes.add(Capsule3d {
        radius: 0.04,
        half_length: axis_length * 0.5f32,
    });
    let cone_mesh = meshes.add(cone::Cone {
        height: 0.25,
        radius: 0.10,
        ..Default::default()
    });
    commands
        .spawn((vector_type, Transform::default(), Visibility::Visible))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(arrow_tail_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: vector_type.color(),
                    ..default()
                })),
                Transform::from_translation(Vec3::new(0.0, axis_length / 2.0, 0.0)),
            ));
            parent.spawn((
                Mesh3d(cone_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: vector_type.color(),
                    ..default()
                })),
                Transform::from_translation(Vec3::new(0.0, axis_length, 0.0)),
            ));
        });
}

pub fn setup_debug_vectors(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_arrow_vector(
        commands.reborrow(),
        WhichDebugVector::VerticalVector,
        &mut meshes,
        &mut materials,
    );
    spawn_arrow_vector(
        commands.reborrow(),
        WhichDebugVector::PlaneNormal,
        &mut meshes,
        &mut materials,
    );
    spawn_arrow_vector(
        commands.reborrow(),
        WhichDebugVector::CursorVector,
        &mut meshes,
        &mut materials,
    );
    spawn_arrow_vector(
        commands.reborrow(),
        WhichDebugVector::PickingRay,
        &mut meshes,
        &mut materials,
    );

    spawn_arrow_vector(
        commands.reborrow(),
        WhichDebugVector::CrossPlaneNormal,
        &mut meshes,
        &mut materials,
    );
    commands
        .spawn((
            WhichDebugVector::CrossPlane,
            Transform::default(),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.0, 0.01, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WhichDebugVector::CrossPlane.color(),
                    ..default()
                })),
                Wireframe,
            ));
        });

    commands
        .spawn((
            WhichDebugVector::PlaneOrigin,
            Transform::default(),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.0, 0.01, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WhichDebugVector::PlaneOrigin.color(),
                    ..default()
                })),
                Wireframe,
            ));
        });

    commands
        .spawn((
            WhichDebugVector::CursorPlaneIntersection,
            Transform::default(),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(meshes.add(Sphere::new(0.05))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WhichDebugVector::CursorPlaneIntersection.color(),
                    ..default()
                })),
            ));
        });
}

/// Update the transform of the debug mesh (the vector we are trying to visualize)
/// from the resource of DebugVectors
pub fn update_debug_vectors(
    vectors: Res<DebugVectors>,
    query: Query<(&WhichDebugVector, &mut Transform)>,
) {
    for (vector, mut transform) in query {
        match vector {
            WhichDebugVector::VerticalVector => {
                let local_forward = Vec3::Y;
                transform.rotation =
                    Quat::from_rotation_arc(local_forward, vectors.vertical_vector);
            }
            WhichDebugVector::CursorPlaneIntersection => {
                transform.translation = vectors.cursor_plane_intersection;
            }
            WhichDebugVector::CursorVector => {
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.cursor_vector);
            }
            WhichDebugVector::PickingRay => {
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, *vectors.ray.direction);
            }
            WhichDebugVector::PlaneOrigin => {
                // the original first plane
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.plane_normal);
                transform.translation = vectors.plane_origin;
            }
            WhichDebugVector::PlaneNormal => {
                //let local_forward = transform.up();
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.plane_normal);
                transform.translation = vectors.plane_origin;
            }
            WhichDebugVector::CrossPlane => {
                let local_forward = Vec3::Z;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.plane_normal);
                transform.translation = vectors.plane_origin;
            }
            WhichDebugVector::CrossPlaneNormal => {
                let local_forward = Vec3::Y;
                let rotation = Quat::from_rotation_arc(local_forward, vectors.cross_plane_normal);
                let mut new_transform = Transform::from_translation(vectors.plane_origin);
                new_transform.rotation = rotation;
                //transform.rotate_local_x(90_f32.to_radians());
                *transform = new_transform;
            }
        }
    }
}
