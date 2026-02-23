use crate::gizmo::TransformGizmo;
use bevy::{color::palettes::css::*, pbr::wireframe::Wireframe, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPrimaryContextPass},
    egui::{self, Color32, RichText},
};

use crate::mesh::cone;

pub struct DebugVectorsPlugin;

impl Plugin for DebugVectorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_debug_vectors)
            .insert_resource(DebugVectors::default())
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
    pub translation_axis: Vec3,
    pub vertical_vector: Vec3,
    pub plane_normal: Vec3,
    pub picking_ray: Ray3d,
    pub plane_origin: Vec3,           // initial click
    pub ray_plane_intersection: Vec3, // drag location
    pub cursor_vector: Vec3,
    pub cross_plane: Vec3,
    pub cross_plane_normal: Vec3,
    pub signed_distance: f32,
}

impl Default for DebugVectors {
    fn default() -> Self {
        Self {
            translation_axis: Default::default(),
            vertical_vector: Default::default(),
            plane_normal: Default::default(),
            picking_ray: Ray3d {
                origin: Vec3::default(),
                direction: Dir3::new(Vec3::new(1.0, 1.0, 0.0).normalize()).unwrap(),
            },
            plane_origin: Default::default(),
            ray_plane_intersection: Default::default(),
            cursor_vector: Default::default(),
            cross_plane: Default::default(),
            cross_plane_normal: Default::default(),
            signed_distance: Default::default(),
        }
    }
}

impl DebugVectors {
    fn value(&self, which: WhichDebugVector) -> Vec3 {
        match which {
            WhichDebugVector::AxisVector => self.translation_axis,
            WhichDebugVector::VerticalVector => self.vertical_vector,
            WhichDebugVector::PlaneNormal => self.plane_normal,
            WhichDebugVector::PlaneOrigin => self.plane_origin,
            WhichDebugVector::RayPlaneIntersection => self.ray_plane_intersection,
            WhichDebugVector::CursorVector => self.cursor_vector,
            WhichDebugVector::PickingRay => *self.picking_ray.direction,
            //WhichDebugVector::CrossPlane => self.cross_plane,
            //WhichDebugVector::CrossPlaneNormal => self.cross_plane_normal,
        }
    }

    pub const VALUES: [WhichDebugVector; 6] = [
        WhichDebugVector::VerticalVector,
        WhichDebugVector::PlaneNormal,
        WhichDebugVector::PlaneOrigin,
        WhichDebugVector::RayPlaneIntersection,
        WhichDebugVector::CursorVector,
        WhichDebugVector::PickingRay,
        //WhichDebugVector::CrossPlane,
        //WhichDebugVector::CrossPlaneNormal,
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
            AxisVector => PURPLE.into(),
            VerticalVector => AQUA.into(),
            PlaneNormal => RED.into(),
            PlaneOrigin => YELLOW.into(),
            RayPlaneIntersection => LIME.into(),
            CursorVector => WHITE.into(),
            PickingRay => ORANGE.into(),
        }
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub enum WhichDebugVector {
    AxisVector,
    VerticalVector,
    PlaneNormal,
    PlaneOrigin, // initial click // first original plane, plane rotates to stay on plane with 3d drag
    RayPlaneIntersection, // drag location
    CursorVector,
    PickingRay,
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

        let interesting = [
            WhichDebugVector::VerticalVector,
            WhichDebugVector::CursorVector,
            WhichDebugVector::PlaneNormal,
            WhichDebugVector::PlaneOrigin,
            WhichDebugVector::PickingRay,
            WhichDebugVector::RayPlaneIntersection,
            //WhichDebugVector::CrossPlaneNormal,
        ];
        for item in interesting {
            let mut text = RichText::new(format!("{:?}\n{:.2}", item, vectors.value(item)))
                .color(item.egui_color());
            text = text.size(25.0);
            ui.label(text);
        }
        let mut text = RichText::new(format!("Signed distance {:.2}", vectors.signed_distance));
        text = text.size(25.0);
        ui.label(text);
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
        WhichDebugVector::AxisVector,
        &mut meshes,
        &mut materials,
    );
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

    /*
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
    */

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
            WhichDebugVector::RayPlaneIntersection,
            Transform::default(),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(meshes.add(Sphere::new(0.05))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WhichDebugVector::RayPlaneIntersection.color(),
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
    let axis_length = 1.3; // from mesh/mod.rs

    for (vector, mut transform) in query {
        match vector {
            WhichDebugVector::AxisVector => {
                let local_forward = Vec3::Y;
                transform.rotation =
                    Quat::from_rotation_arc(local_forward, vectors.translation_axis);
            }
            WhichDebugVector::VerticalVector => {
                let local_forward = Vec3::Y;
                transform.rotation =
                    Quat::from_rotation_arc(local_forward, vectors.vertical_vector);
            }
            WhichDebugVector::RayPlaneIntersection => {
                transform.translation = vectors.ray_plane_intersection;
            }
            WhichDebugVector::CursorVector => {
                let local_forward = Vec3::NEG_Y;
                transform.translation = vectors.ray_plane_intersection;
                let norm = vectors.cursor_vector.normalize();
                transform.rotation = Quat::from_rotation_arc(local_forward, norm);

                // let cursor_vector: Vec3 = ray_plane_intersection - plane_origin;
                // vectors.ray_plane_intersection - vectors.plane_origin;
                let len = vectors.cursor_vector.length();
                let mult = len / axis_length;
                transform.scale.y = mult;
            }
            WhichDebugVector::PickingRay => {
                let local_forward = Vec3::Y;
                transform.rotation =
                    Quat::from_rotation_arc(local_forward, *vectors.picking_ray.direction);
                let diff = vectors.ray_plane_intersection - vectors.picking_ray.origin;
                let len = diff.length();
                let mult = len / axis_length;
                transform.scale.y = mult;
                transform.translation = vectors.picking_ray.origin;
            }
            WhichDebugVector::PlaneOrigin => {
                // the original first plane
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.plane_normal);
                transform.translation = vectors.plane_origin;
            }
            WhichDebugVector::PlaneNormal => {
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.plane_normal);
                transform.translation = vectors.plane_origin;
            } /*
                          WhichDebugVector::CrossPlane => {
                              let local_forward = Vec3::Y;
                              transform.rotation =
                                  Quat::from_rotation_arc(local_forward, vectors.cross_plane_normal);
                              transform.translation = vectors.plane_origin;
                          }
                          WhichDebugVector::CrossPlaneNormal => {
                              let local_forward = Vec3::Y;
                              transform.rotation =
                                  Quat::from_rotation_arc(local_forward, vectors.cross_plane_normal);
                              transform.translation = vectors.plane_origin;
                          }
              */
        }
    }
}
