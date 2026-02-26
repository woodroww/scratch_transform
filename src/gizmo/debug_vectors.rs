use crate::gizmo::TransformGizmo;
use bevy::{color::palettes::css::*, pbr::wireframe::Wireframe, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPrimaryContextPass},
    egui::{self, Color32, RichText, Ui},
};

pub struct DebugVectorsPlugin;

pub const AXIS_LENGTH: f32 = 1.0;

impl Plugin for DebugVectorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_debug_vectors, setup_rotation_vectors))
            .add_systems(Update, hide_drag_vectors)
            .insert_resource(DebugVectors::default())
            .insert_resource(RotateDebugVectors::default())
            .add_systems(EguiPrimaryContextPass, debug_ui)
            .add_systems(
                Update,
                update_debug_vectors.run_if(resource_changed::<DebugVectors>),
            )
            .add_systems(
                Update,
                update_rotation_vectors.run_if(resource_changed::<RotateDebugVectors>),
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
    pub signed_distance: f32,
}

#[derive(Resource)]
pub struct RotateDebugVectors {
    pub cam_forward: Vec3,
    pub gizmo_forward: Vec3,
    pub picking_ray: Ray3d,
    pub arccosine: f32,
    /*
    pub gizmo_initial_transform: Transform,
    pub rotation_axis: Vec3,
    pub vertical_vector: Vec3,
    pub plane_normal: Vec3,
    pub plane_origin: Vec3,           // initial click
    pub ray_plane_intersection: Vec3, // drag location
    pub dot: f32,
    pub det: f32,
    pub angle: f32,
    */
}

impl Default for RotateDebugVectors {
    fn default() -> Self {
        Self {
            cam_forward: Vec3::default(),
            gizmo_forward: Vec3::default(),
            picking_ray: Ray3d {
                origin: Vec3::default(),
                direction: Dir3::new(Vec3::new(1.0, 1.0, 0.0).normalize()).unwrap(),
            },
            arccosine: 0.0,
            /*
                        gizmo_initial_transform: Transform::default(),
                        rotation_axis: Vec3::default(),
                        vertical_vector: Vec3::default(),
                        plane_normal: Vec3::default(),
                        plane_origin: Vec3::default(),
                        ray_plane_intersection: Vec3::default(),
                        dot: 0.0,
                        det: 0.0,
                        angle: 0.0,
            */
        }
    }
}

impl RotateDebugVectors {
    fn value(&self, which: WhichRotateVector) -> Vec3 {
        match which {
            WhichRotateVector::CamForward => self.cam_forward,
            WhichRotateVector::GizmoForward => self.gizmo_forward,
            WhichRotateVector::PickingRay => *self.picking_ray.direction,
            WhichRotateVector::ArcCosine => Vec3::new(self.arccosine, self.arccosine.to_degrees(), 0.0),
            /*
                        WhichRotateVector::RotationAxis => self.rotation_axis,
                        WhichRotateVector::VerticalVector => self.vertical_vector,
                        WhichRotateVector::PlaneNormal => self.plane_normal,
                        WhichRotateVector::PlaneOrigin => self.plane_origin,
                        WhichRotateVector::RayPlaneIntersection => self.ray_plane_intersection,
                        WhichRotateVector::Dot => Vec3::splat(self.dot.to_degrees()),
                        WhichRotateVector::Det => Vec3::splat(self.det.to_degrees()),
                        WhichRotateVector::Angle => Vec3::splat(self.angle.to_degrees()),
            */
        }
    }
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
        }
    }

    pub const VALUES: [WhichDebugVector; 6] = [
        WhichDebugVector::VerticalVector,
        WhichDebugVector::PlaneNormal,
        WhichDebugVector::PlaneOrigin,
        WhichDebugVector::RayPlaneIntersection,
        WhichDebugVector::CursorVector,
        WhichDebugVector::PickingRay,
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
    PickingRay,
    AxisVector,
    VerticalVector,
    PlaneNormal,
    PlaneOrigin,          // initial click // plane rotates to stay on plane with 3d drag
    RayPlaneIntersection, // drag location
    CursorVector,
}

#[derive(Component, Copy, Clone, Debug)]
pub enum WhichRotateVector {
    CamForward,
    GizmoForward,
    PickingRay,
    ArcCosine,
    /*
    RotationAxis,
    VerticalVector,
    PlaneNormal,
    PlaneOrigin,          // initial click // plane rotates to stay on plane with 3d drag
    RayPlaneIntersection, // drag location
    Dot,
    Det,
    Angle,
    */
}

impl WhichRotateVector {
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
        use WhichRotateVector::*;
        match self {
            CamForward => PURPLE.into(),
            GizmoForward => YELLOW.into(),
            PickingRay => GREEN.into(),
            ArcCosine => AQUA.into(),
            /*
                        RotationAxis => AQUA.into(),
                        VerticalVector => GREEN.into(),
                        PlaneNormal => RED.into(),
                        PlaneOrigin => YELLOW.into(),
                        RayPlaneIntersection => LIME.into(),
                        Dot => WHITE.into(),
                        Det => WHITE.into(),
                        Angle => ORANGE.into(),
            */
        }
    }
}

fn rotate_ui(
    ui: &mut Ui,
    _gizmo_transform: &Transform,
    _gizmo: &TransformGizmo,
    vectors: &RotateDebugVectors,
) {
    ui.label("Rotate");
    let interesting = [
        WhichRotateVector::CamForward,
        WhichRotateVector::GizmoForward,
        WhichRotateVector::PickingRay,
        WhichRotateVector::ArcCosine,
        /*
                WhichRotateVector::Angle,
                WhichRotateVector::Det,
                WhichRotateVector::Dot,
        */
    ];
    for item in interesting {
        let mut text = RichText::new(format!("{:?}\n{:.2}", item, vectors.value(item)))
            .color(item.egui_color());
        text = text.size(25.0);
        ui.label(text);
    }
}

fn drag_ui(
    ui: &mut Ui,
    gizmo_transform: &Transform,
    _gizmo: &TransformGizmo,
    vectors: &DebugVectors,
) {
    ui.label(format!(
        "gizmo.............{:.3}",
        gizmo_transform.translation
    ));

    let interesting = [
        WhichDebugVector::VerticalVector,
        WhichDebugVector::CursorVector,
        WhichDebugVector::PlaneNormal,
        WhichDebugVector::PlaneOrigin,
        WhichDebugVector::PickingRay,
        WhichDebugVector::RayPlaneIntersection,
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
}

pub fn debug_ui(
    mut contexts: EguiContexts,
    axis_vectors: Res<DebugVectors>,
    rotate_vectors: Res<RotateDebugVectors>,
    gizmo_query: Query<(&Transform, &TransformGizmo)>,
) {
    let ctx = contexts.ctx_mut().unwrap();
    egui::Window::new("DebugVectors").show(ctx, |ui| {
        if let Ok((transform, gizmo)) = gizmo_query.single() {
            if let Some(interaction) = gizmo.current_interaction {
                match interaction {
                    crate::gizmo::TransformGizmoInteraction::TranslateAxis {
                        original: _,
                        axis: _,
                    } => {
                        drag_ui(ui, transform, gizmo, &axis_vectors);
                    }
                    crate::gizmo::TransformGizmoInteraction::TranslatePlane {
                        original: _,
                        normal: _,
                    } => {}
                    crate::gizmo::TransformGizmoInteraction::RotateAxis {
                        original: _,
                        axis: _,
                    } => {
                        rotate_ui(ui, transform, gizmo, &rotate_vectors);
                    }
                    crate::gizmo::TransformGizmoInteraction::ScaleAxis {
                        original: _,
                        axis: _,
                    } => {}
                }
            }
        } else {
            warn!("why no single gizmo_query?");
        }
    });
}

pub fn spawn_arrow_vector<T: Component>(
    mut commands: Commands,
    vector_type: T,
    color: Color,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let arrow_tail_mesh = meshes.add(Capsule3d {
        radius: 0.04,
        half_length: AXIS_LENGTH * 0.5f32,
    });
    let cone_mesh = meshes.add(Cone {
        height: 0.25,
        radius: 0.10,
    });
    commands
        .spawn((vector_type, Transform::default(), Visibility::Visible))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(arrow_tail_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(0.0, AXIS_LENGTH / 2.0, 0.0)),
            ));
            parent.spawn((
                Mesh3d(cone_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(0.0, AXIS_LENGTH, 0.0)),
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
        WhichDebugVector::AxisVector.color(),
        &mut meshes,
        &mut materials,
    );
    spawn_arrow_vector(
        commands.reborrow(),
        WhichDebugVector::VerticalVector,
        WhichDebugVector::VerticalVector.color(),
        &mut meshes,
        &mut materials,
    );
    spawn_arrow_vector(
        commands.reborrow(),
        WhichDebugVector::PlaneNormal,
        WhichDebugVector::PlaneNormal.color(),
        &mut meshes,
        &mut materials,
    );
    spawn_arrow_vector(
        commands.reborrow(),
        WhichDebugVector::CursorVector,
        WhichDebugVector::CursorVector.color(),
        &mut meshes,
        &mut materials,
    );
    spawn_arrow_vector(
        commands.reborrow(),
        WhichDebugVector::PickingRay,
        WhichDebugVector::PickingRay.color(),
        &mut meshes,
        &mut materials,
    );

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

pub fn setup_rotation_vectors(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_arrow_vector(
        commands.reborrow(),
        WhichRotateVector::CamForward,
        WhichRotateVector::CamForward.color(),
        &mut meshes,
        &mut materials,
    );
    spawn_arrow_vector(
        commands.reborrow(),
        WhichRotateVector::GizmoForward,
        WhichRotateVector::GizmoForward.color(),
        &mut meshes,
        &mut materials,
    );
    spawn_arrow_vector(
        commands.reborrow(),
        WhichRotateVector::PickingRay,
        WhichRotateVector::PickingRay.color(),
        &mut meshes,
        &mut materials,
    );
}

pub fn update_rotation_vectors(
    vectors: Res<RotateDebugVectors>,
    query: Query<(&WhichRotateVector, &mut Transform)>,
) {
    for (vector, mut transform) in query {
        match vector {
            WhichRotateVector::CamForward => {
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.cam_forward);
            }
            WhichRotateVector::GizmoForward => {
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.gizmo_forward);
            }
            WhichRotateVector::PickingRay => {
                let local_forward = Vec3::Y;
                transform.rotation =
                    Quat::from_rotation_arc(local_forward, *vectors.picking_ray.direction);
            }
            WhichRotateVector::ArcCosine => {},
        }
    }
}

pub fn hide_drag_vectors(
    gizmo_query: Query<&TransformGizmo>,
    mut axis_query: Query<
        (&WhichDebugVector, &mut Transform, &mut Visibility),
        Without<WhichRotateVector>,
    >,
    mut rotate_query: Query<
        (&WhichRotateVector, &mut Transform, &mut Visibility),
        Without<WhichDebugVector>,
    >,
) {
    let Ok(gizmo) = gizmo_query.single() else {
        return;
    };
    if let Some(interaction) = gizmo.current_interaction {
        use crate::gizmo::TransformGizmoInteraction::*;
        match interaction {
            TranslateAxis { .. } => {
                for (_vector, _transform, mut visibility) in axis_query.iter_mut() {
                    *visibility = Visibility::Inherited;
                }
            }
            TranslatePlane { .. } => {
                for (_vector, _transform, mut visibility) in axis_query.iter_mut() {
                    *visibility = Visibility::Inherited;
                }
            }
            RotateAxis { .. } => {
                for (_vector, _transform, mut visibility) in axis_query.iter_mut() {
                    *visibility = Visibility::Hidden;
                }
            }
            ScaleAxis { .. } => {
                for (_vector, _transform, mut visibility) in axis_query.iter_mut() {
                    *visibility = Visibility::Hidden;
                }
            }
        }
        match interaction {
            TranslateAxis { .. } => {
                for (_vector, _transform, mut visibility) in rotate_query.iter_mut() {
                    *visibility = Visibility::Hidden;
                }
            }
            TranslatePlane { .. } => {
                for (_vector, _transform, mut visibility) in rotate_query.iter_mut() {
                    *visibility = Visibility::Hidden;
                }
            }
            RotateAxis { .. } => {
                for (_vector, _transform, mut visibility) in rotate_query.iter_mut() {
                    *visibility = Visibility::Inherited;
                }
            }
            ScaleAxis { .. } => {
                for (_vector, _transform, mut visibility) in rotate_query.iter_mut() {
                    *visibility = Visibility::Inherited;
                }
            }
        }
    }
}

/// Update the transform of the debug mesh (the vector we are trying to visualize)
/// from the resource of DebugVectors
pub fn update_debug_vectors(
    vectors: Res<DebugVectors>,
    query: Query<(&WhichDebugVector, &mut Transform, &mut Visibility)>,
) {
    for (vector, mut transform, _) in query {
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
                let mult = len / AXIS_LENGTH;
                transform.scale.y = mult;
            }
            WhichDebugVector::PickingRay => {
                let local_forward = Vec3::Y;
                transform.rotation =
                    Quat::from_rotation_arc(local_forward, *vectors.picking_ray.direction);
                let diff = vectors.ray_plane_intersection - vectors.picking_ray.origin;
                let len = diff.length();
                let mult = len / AXIS_LENGTH;
                transform.scale.y = mult;
                transform.translation = vectors.picking_ray.origin;
            }
            WhichDebugVector::PlaneOrigin => {
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.plane_normal);
                transform.translation = vectors.plane_origin;
            }
            WhichDebugVector::PlaneNormal => {
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.plane_normal);
                transform.translation = vectors.plane_origin;
            }
        }
    }
}
