use bevy::{color::palettes::{basic::AQUA, css::{FUCHSIA, GREEN, LIME, PURPLE, RED, TEAL, YELLOW}}, picking::backend::PointerHits, prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPrimaryContextPass},
    egui::{self, Color32, RichText},
};

use crate::{gizmo_material::GizmoMaterial, mesh::cone};

#[derive(Component)]
pub struct GizmoPickSource;

#[derive(Component, Debug)]
pub struct PickSelection {
    pub is_selected: bool,
}

/// Marks the current active gizmo interaction
#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub enum TransformGizmoInteraction {
    TranslateAxis { original: Vec3, axis: Vec3 },
    TranslatePlane { original: Vec3, normal: Vec3 },
    RotateAxis { original: Vec3, axis: Vec3 },
    ScaleAxis { original: Vec3, axis: Vec3 },
}

#[derive(Component)]
pub struct InitialTransform {
    transform: Transform,
    rotation_offset: Vec3,
}

#[derive(Default, PartialEq, Component)]
pub struct TransformGizmo {
    current_interaction: Option<TransformGizmoInteraction>,
    // Point in space where mouse-gizmo interaction started (on mouse down), used to compare how
    // much total dragging has occurred without accumulating error across frames.
    drag_start: Option<Vec3>,
    // Initial transform of the gizmo
    initial_transform: Option<Transform>,
}

pub fn ray_from_screenspace(
    cursor_pos_screen: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
) -> Option<Ray3d> {
    let mut viewport_pos = cursor_pos_screen;
    if let Some(viewport) = &camera.viewport {
        viewport_pos -= viewport.physical_position.as_vec2() / window.scale_factor();
    }
    camera
        .viewport_to_world(camera_transform, viewport_pos)
        .ok()
}

fn intersect_plane(ray: Ray3d, plane_normal: Vec3, plane_origin: Vec3) -> Option<Vec3> {
    // assuming vectors are all normalized
    let denominator = ray.direction.dot(plane_normal);
    if denominator.abs() > f32::EPSILON {
        let point_to_point = plane_origin - ray.origin;
        let intersect_dist = plane_normal.dot(point_to_point) / denominator;
        let intersect_position = ray.direction * intersect_dist + ray.origin;
        Some(intersect_position)
    } else {
        None
    }
}

pub struct TransformGizmoPlugin;

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (crate::mesh::spawn_gizmo, setup_debug_vectors))
            .insert_resource(DebugVectors::default())
            .add_systems(EguiPrimaryContextPass, debug_ui)
            .add_systems(
                Update,
                update_debug_vectors.run_if(resource_changed::<DebugVectors>),
            )
            .add_plugins(MaterialPlugin::<GizmoMaterial>::default())
            .add_observer(on_component_added);
    }
}

fn on_component_added(event: On<Add, TransformGizmo>) {
    println!(
        "My marker component was added to entity: {:?}",
        event.entity
    );
}

pub fn debug_print_hits(
    msg_i: usize,
    hit: &PointerHits,
    transform_query: &Query<
        (&TransformGizmoInteraction, Option<&ChildOf>, &mut Transform),
        Without<TransformGizmo>,
    >,
    drag: &On<Pointer<DragStart>>,
) {
    for (i, pick) in hit.picks.iter().enumerate() {
        println!("msg_i: {}, pick: {}", msg_i, i);
        println!("\tposition: {:?}", pick.1.position);
        println!("\tnormal:   {:?}", pick.1.normal);
        println!("\tdepth:    {:?}", pick.1.depth);
        match transform_query.get(pick.0) {
            Ok((interaction, _child_of, _transform)) => {
                if drag.entity == pick.0 {
                    println!("\tGIZMO:    {:?}", interaction);
                } else {
                    println!("\tgizmo:    {:?}", interaction);
                }
            }
            Err(_) => {
                println!("\tgizmo:    None");
            }
        }
    }
}

pub fn click_axis(
    drag: On<Pointer<DragStart>>,
    mut commands: Commands,
    transform_query: Query<
        (&TransformGizmoInteraction, Option<&ChildOf>, &mut Transform),
        Without<TransformGizmo>,
    >,
    mut gizmo: Query<(&GlobalTransform, &Transform, &mut TransformGizmo)>,
    mut hit_reader: MessageReader<PointerHits>,
) {
    debug_assert_eq!(transform_query.iter().len(), 13);

    let mut min_depth = f32::MAX;
    let mut min_entity = None;
    let mut min_data = None;
    for hit_message in hit_reader.read() {
        //debug_print_hits(msg_i, hit_message, &transform_query, &drag);
        for (_hit_i, data) in
            hit_message
                .picks
                .iter()
                .enumerate()
                .filter_map(|(hit_i, (entity, data))| {
                    if *entity == drag.entity {
                        Some((hit_i, data))
                    } else {
                        None
                    }
                })
        {
            if data.depth != 0.0 && data.depth < min_depth {
                min_depth = data.depth;
                min_entity = Some(drag.entity);
                min_data = Some(data);
            }
        }
    }

    let position = drag.pointer_location.position;

    info!("click position: {:#?}", position);
    println!("min entity: {:?}", min_entity);
    debug_assert_eq!(min_entity, Some(drag.entity));
    println!("min data:   {:?}", min_data);

    // if there are multiple gizmos allowed we're going to have to find the one clicked
    // but for now this
    let Ok((main_global_transform, main_transform, mut transform_gizmo)) = gizmo.single_mut()
    else {
        warn!("getting main gizmo error");
        return;
    };

    let Ok((interaction, child_of, transform)) = transform_query.get(drag.entity) else {
        warn!("transform_query couldn't find entity from click");
        return;
    };

    transform_gizmo.current_interaction = Some(*interaction);
    transform_gizmo.drag_start = Some(min_data.unwrap().position.unwrap());
    transform_gizmo.initial_transform = Some(*main_transform);
}

pub fn drag_axis(
    drag: On<Pointer<Drag>>,
    pick_cam: Query<(&Camera, &GlobalTransform), With<GizmoPickSource>>,
    windows: Query<&mut Window, With<PrimaryWindow>>,
    mut gizmo_query: Query<(&mut Transform, &GlobalTransform, &mut TransformGizmo)>,
    mut debug_vectors: ResMut<DebugVectors>,
    transform_query: Query<
        (
            &PickSelection,
            Option<&ChildOf>,
            &mut Transform,
        ),
        Without<TransformGizmo>,
    >,
    parent_query: Query<&GlobalTransform>,
) {
    let Ok((mut gizmo_local_transform, _gizmo_global_transform, gizmo)) =
        gizmo_query.single_mut()
    else {
        let len = gizmo_query.iter().len();
        warn!("error gizmo_query.single_mut() len: {}", len);
        return;
    };
    let Some(initial_transform) = gizmo.initial_transform else {
        warn!("no gizmo.initial_transform");
        return;
    };

    let Ok(window) = windows.single() else {
        warn!("no window");
        return;
    };

    let Some((picking_camera, global_cam_tran)) = pick_cam.iter().last() else {
        warn!("Not exactly one picking camera.");
        return;
    };

    let pointer = drag.pointer_location.position;
    let Some(picking_ray) = ray_from_screenspace(pointer, picking_camera, global_cam_tran, window)
    else {
        warn!("error creating ray");
        return;
    };

    let Some(gizmo_origin) = gizmo.drag_start else {
        warn!("no gizmo.drag_start");
        return;
    };

    let Some(interaction) = gizmo.current_interaction else {
        warn!("no gizmo.current_interaction");
        return;
    };

    let TransformGizmoInteraction::TranslateAxis { original: _, axis } = interaction else {
        warn!("what? interaction is not a TranslateAxis!");
        return;
    };
    let vertical_vector = picking_ray.direction.cross(axis).normalize();
    let plane_normal = axis.cross(vertical_vector).normalize();
    let plane_origin = gizmo_origin;
    let Some(cursor_plane_intersection) = intersect_plane(picking_ray, plane_normal, plane_origin)
    else {
        warn!("what? None cursor_plane_intersection");
        return;
    };
    let cursor_vector: Vec3 = cursor_plane_intersection - plane_origin;

    // initial click PlaneOrigin vectors.plane_origin
    // drag location CursorPlaneIntersection  vectors.cursor_plane_intersection
    let new_translation = initial_transform.translation + cursor_vector * axis;

    *debug_vectors = DebugVectors {
        vertical_vector,
        plane_normal,
        plane_origin,
        cursor_plane_intersection,
        cursor_vector,
    };

    gizmo_local_transform.translation = new_translation;
}

#[derive(Resource, Default)]
pub struct DebugVectors {
    vertical_vector: Vec3,
    plane_normal: Vec3,
    plane_origin: Vec3,
    cursor_plane_intersection: Vec3,
    cursor_vector: Vec3,
}

impl DebugVectors {
    fn value(self, which: WhichDebugVector) -> Vec3 {
        match which {
            WhichDebugVector::VerticalVector => self.vertical_vector,
            WhichDebugVector::PlaneNormal => self.plane_normal,
            WhichDebugVector::PlaneOrigin => self.plane_origin,
            WhichDebugVector::CursorPlaneIntersection => self.cursor_plane_intersection,
            WhichDebugVector::CursorVector => self.cursor_vector,
        }
    }
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
            CursorVector => PURPLE.into(),
        }
    }
}

#[derive(Component, Copy, Clone)]
pub enum WhichDebugVector {
    VerticalVector,
    PlaneNormal,
    PlaneOrigin,
    CursorPlaneIntersection,
    CursorVector,
}

fn debug_ui(
    mut contexts: EguiContexts,
    vectors: Res<DebugVectors>,
    gizmo_query: Query<&Transform, With<TransformGizmo>>,
) {
    let ctx = contexts.ctx_mut().unwrap();
    egui::Window::new("DebugVectors").show(ctx, |ui| {
        if let Ok(gizmo_transform) = gizmo_query.single() {
            ui.label(format!("gizmo.............{}", gizmo_transform.translation));
        } else {
            ui.label("gizmo.............error");
        }
        // initial click
        let color = WhichDebugVector::PlaneOrigin.egui_color();
        let color_txt = RichText::new(format!("plane_origin......{}", vectors.plane_origin)).color(color);
        ui.label(color_txt);

        // drag location
        let color = WhichDebugVector::CursorPlaneIntersection.egui_color();
        let color_txt = RichText::new(format!("cursor_plane_inte.{}", vectors.cursor_plane_intersection)).color(color);
        ui.label(color_txt);

        let color = WhichDebugVector::VerticalVector.egui_color();
        let color_txt = RichText::new(format!("vertical_vector...{}", vectors.vertical_vector)).color(color);
        ui.label(color_txt);

        let color = WhichDebugVector::PlaneNormal.egui_color();
        let color_txt = RichText::new(format!("plane_normal......{}", vectors.plane_normal)).color(color);
        ui.label(color_txt);

        let color = WhichDebugVector::CursorVector.egui_color();
        let color_txt = RichText::new(format!("cursor_vector.....{}", vectors.cursor_vector)).color(color);
        ui.label(color_txt);
    });
}

fn spawn_arrow_vector(
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
        .spawn((
            vector_type,
            Transform::default(),
            Visibility::Visible,
        ))
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

fn setup_debug_vectors(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_arrow_vector(commands.reborrow(), WhichDebugVector::VerticalVector, &mut meshes, &mut materials);
    spawn_arrow_vector(commands.reborrow(), WhichDebugVector::PlaneNormal, &mut meshes, &mut materials);
    spawn_arrow_vector(commands.reborrow(), WhichDebugVector::CursorVector, &mut meshes, &mut materials);

    commands
        .spawn((
            WhichDebugVector::PlaneOrigin,
            Transform::default(),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(meshes.add(Sphere::new(0.05))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WhichDebugVector::PlaneOrigin.color(),
                    ..default()
                })),
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


fn update_debug_vectors(
    vectors: Res<DebugVectors>,
    query: Query<(&WhichDebugVector, &mut Transform)>,
) {
    for (vector, mut transform) in query {
        match vector {
            WhichDebugVector::VerticalVector => {
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.vertical_vector);
            }
            WhichDebugVector::PlaneNormal => {
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.plane_normal);
            }
            WhichDebugVector::PlaneOrigin => {
                transform.translation = vectors.plane_origin;
            }
            WhichDebugVector::CursorPlaneIntersection => {
                transform.translation = vectors.cursor_plane_intersection;
            }
            WhichDebugVector::CursorVector => {
                let local_forward = Vec3::Y;
                transform.rotation = Quat::from_rotation_arc(local_forward, vectors.cursor_vector);
            }
        }
    }
}

pub fn drag_axis_end(
    drag: On<Pointer<DragEnd>>,
    mut commands: Commands,
    transform_query: Query<
        (
            Entity,
            &TransformGizmoInteraction,
            Option<&ChildOf>,
            &mut Transform,
            &InitialTransform,
        ),
        Without<TransformGizmo>,
    >,
) {
    for (entity, _interaction, _child_of, _transform, _initial_transform) in transform_query {
        commands.entity(entity).remove::<InitialTransform>();
    }
}

pub fn click_rotate(drag: On<Pointer<DragStart>>) {
    info!("click_rotate");
}

pub fn drag_rotate(drag: On<Pointer<Drag>>) {
    info!("drag_rotate");
}

pub fn click_plane(drag: On<Pointer<DragStart>>) {
    info!("click_plane");
}

pub fn drag_plane(drag: On<Pointer<Drag>>) {
    info!("drag_plane");
}
