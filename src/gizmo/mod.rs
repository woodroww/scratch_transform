use bevy::{picking::hover::PickingInteraction, prelude::*, window::PrimaryWindow};

use crate::gizmo_material::GizmoMaterial;

#[derive(Component)]
pub struct GizmoPickSource;

#[derive(Component)]
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
    origin_drag_start: Option<Vec3>,
    // Initial transform of the gizmo
    initial_transform: Option<GlobalTransform>,
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
        app.add_systems(Startup, crate::mesh::spawn_gizmo)
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

pub fn click_axis(drag: On<Pointer<DragStart>>) {
    info!("click_axis");
}

pub fn drag_axis(
    drag: On<Pointer<Drag>>,
    pick_cam: Query<(&Camera, &GlobalTransform), With<GizmoPickSource>>,
    query: Query<&TransformGizmoInteraction>,
    windows: Query<&mut Window, With<PrimaryWindow>>,
    mut gizmo_query: Query<(&mut Transform, &GlobalTransform, &mut TransformGizmo)>,
    mut transform_query: Query<
        (
            &PickSelection,
            Option<&ChildOf>,
            &mut Transform,
            &InitialTransform,
        ),
        Without<TransformGizmo>,
    >,
    parent_query: Query<&GlobalTransform>,
) {
    info!("drag_axis");

    let Ok((mut gizmo_local_transform, gizmo_transform, mut gizmo)) = gizmo_query.single_mut() else {
        let len = gizmo_query.iter().len();
        warn!("error gizmo_query.single_mut() len: {}", len);
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

    let Ok(interaction) = query.get(drag.entity) else {
        warn!("what? no interaction!");
        return;
    };

    let gizmo_origin = match gizmo.origin_drag_start {
        Some(origin) => origin,
        None => {
            let origin = gizmo_transform.translation();
            gizmo.origin_drag_start = Some(origin);
            origin
        }
    };

    let selected_iter = transform_query
        .iter_mut()
        .filter(|(s, ..)| s.is_selected)
        .map(|(_, parent, local_transform, initial_global_transform)| {
            let parent_global_transform = match parent {
                Some(parent) => match parent_query.get(parent.parent()) {
                    Ok(transform) => *transform,
                    Err(_) => GlobalTransform::IDENTITY,
                },
                None => GlobalTransform::IDENTITY,
            };
            let parent_mat = parent_global_transform.to_matrix();
            let inverse_parent = parent_mat.inverse();
            (inverse_parent, local_transform, initial_global_transform)
        });

    let TransformGizmoInteraction::TranslateAxis { original: _, axis } = interaction else {
        warn!("what? interaction is not a TranslateAxis!");
        return;
    };
    let vertical_vector = picking_ray.direction.cross(*axis).normalize();

    let plane_normal = axis.cross(vertical_vector).normalize();
    let plane_origin = gizmo_origin;
    let Some(cursor_plane_intersection) = intersect_plane(picking_ray, plane_normal, plane_origin)
    else {
        warn!("what? None cursor_plane_intersection");
        return;
    };
    let cursor_vector: Vec3 = cursor_plane_intersection - plane_origin;
    let cursor_projected_onto_handle = match &gizmo.drag_start {
        Some(drag_start) => *drag_start,
        None => {
            let handle_vector = axis;
            let cursor_projected_onto_handle =
                cursor_vector.dot(handle_vector.normalize()) * handle_vector.normalize();
            gizmo.drag_start = Some(cursor_projected_onto_handle + plane_origin);
            warn!("what? None cursor_projected_onto_handle");
            return;
        }
    };
    let selected_handle_vec = cursor_projected_onto_handle - plane_origin;
    let new_handle_vec =
        cursor_vector.dot(selected_handle_vec.normalize()) * selected_handle_vec.normalize();
    let translation = new_handle_vec - selected_handle_vec;
    selected_iter.for_each(
        |(inverse_parent, mut local_transform, initial_global_transform)| {
            let new_transform = Transform {
                translation: initial_global_transform.transform.translation + translation,
                rotation: initial_global_transform.transform.rotation,
                scale: initial_global_transform.transform.scale,
            };
            let local = inverse_parent * new_transform.to_matrix();
            local_transform.set_if_neq(Transform::from_matrix(local));
            *gizmo_local_transform = Transform::from_matrix(local);
        },
    );
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
