use bevy::{
    picking::backend::PointerHits, prelude::*, window::PrimaryWindow,
};

use crate::{
    gizmo::debug_vectors::{DebugVectors, DebugVectorsPlugin},
    gizmo_material::GizmoMaterial,
};

pub mod debug_vectors;

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

#[derive(Default, PartialEq, Component)]
pub struct TransformGizmo {
    current_interaction: Option<TransformGizmoInteraction>,
    // Point in space where mouse-gizmo interaction started (on mouse down), used to compare how
    // much total dragging has occurred without accumulating error across frames.
    drag_start: Option<Vec3>,
    // Initial transform of the gizmo
    initial_transform: Transform,
    initial_global_transform: GlobalTransform,
    alignment_rotation: Quat,
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
            .add_plugins(DebugVectorsPlugin)
            .add_plugins(MaterialPlugin::<GizmoMaterial>::default());
    }
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

    let Ok((interaction, _child_of, _transform)) = transform_query.get(drag.entity) else {
        warn!("transform_query couldn't find entity from click");
        return;
    };

    transform_gizmo.current_interaction = Some(*interaction);
    transform_gizmo.drag_start = Some(min_data.unwrap().position.unwrap());
    transform_gizmo.initial_transform = *main_transform;
    transform_gizmo.initial_global_transform = *main_global_transform;
}

pub fn drag_axis(
    drag: On<Pointer<Drag>>,
    pick_cam: Query<(&Camera, &GlobalTransform), With<GizmoPickSource>>,
    windows: Query<&mut Window, With<PrimaryWindow>>,
    mut gizmo_query: Query<(&mut Transform, &GlobalTransform, &mut TransformGizmo)>,
    mut debug_vectors: ResMut<DebugVectors>,
) {
    let Ok((mut gizmo_local_transform, _gizmo_global_transform, gizmo)) = gizmo_query.single_mut()
    else {
        let len = gizmo_query.iter().len();
        warn!("error gizmo_query.single_mut() len: {}", len);
        return;
    };
    let initial_transform = gizmo.initial_transform;
    //let initial_transform = gizmo.initial_global_transform;
    //let rotation_offset = gizmo.alignment_rotation;

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

    let Some(drag_start) = gizmo.drag_start else {
        warn!("no gizmo.drag_start");
        return;
    };

    let Some(interaction) = gizmo.current_interaction else {
        warn!("no gizmo.current_interaction");
        return;
    };

    match interaction {
        TransformGizmoInteraction::TranslateAxis { original: _, axis } => {
            let vertical_vector = picking_ray.direction.cross(axis).normalize();
            let plane_normal = axis.cross(vertical_vector).normalize();
            let plane_origin = drag_start;
            let Some(ray_plane_intersection) =
                intersect_plane(picking_ray, plane_normal, plane_origin)
            else {
                warn!("what? None cursor_plane_intersection");
                return;
            };
            let cursor_vector: Vec3 = ray_plane_intersection - plane_origin;
            let normalized_translation_axis = (initial_transform.rotation * axis).normalize();

            let plane = InfinitePlane3d::new(normalized_translation_axis);
            let isometry = Isometry3d::from_translation(plane_origin);
            // so we needed a signed distance instead of length
            let signed_distance = plane.signed_distance(isometry, ray_plane_intersection);
            //let signed_distance = cursor_vector.length();

            let translation = normalized_translation_axis * signed_distance;
            // if cursor_plane_intersection crosses the plane_origin on the translated axis
            // length sign needs to change
           
            let new_translation = initial_transform.translation + translation;

            *debug_vectors = DebugVectors {
                vertical_vector,
                plane_normal,
                picking_ray,
                plane_origin,
                ray_plane_intersection,
                cursor_vector,
                cross_plane: debug_vectors.cross_plane,
                cross_plane_normal: normalized_translation_axis,
                signed_distance,
            };

            gizmo_local_transform.translation = new_translation;
        }
        TransformGizmoInteraction::TranslatePlane {
            original: _,
            normal,
        } => {
            let plane_origin = drag_start;
            let Some(ray_plane_intersection) =
                intersect_plane(picking_ray, normal, plane_origin)
            else {
                warn!("what? None cursor_plane_intersection");
                return;
            };

            let new_transform = Transform {
                translation: initial_transform.translation + ray_plane_intersection - drag_start,
                rotation: initial_transform.rotation,
                scale: initial_transform.scale,
            };

            *gizmo_local_transform = new_transform;
        }
        TransformGizmoInteraction::RotateAxis { original: _, axis } => {
            let Some(cursor_plane_intersection) =
                intersect_plane(picking_ray, axis.normalize(), drag_start)
            else {
                return;
            };
            let cursor_vector = (cursor_plane_intersection - drag_start).normalize();
            let dot = drag_start.dot(cursor_vector);
            let det = axis.dot(drag_start.cross(cursor_vector));
            let angle = det.atan2(dot);
            let rotation = Quat::from_axis_angle(axis, angle);
            gizmo_local_transform.rotation = rotation;
        }
        TransformGizmoInteraction::ScaleAxis {
            original: _,
            axis: _,
        } => {}
    }
}

pub fn drag_axis_end(
    _drag: On<Pointer<DragEnd>>,
    mut commands: Commands,
    transform_query: Query<
        (
            Entity,
            &TransformGizmoInteraction,
            Option<&ChildOf>,
            &mut Transform,
        ),
        Without<TransformGizmo>,
    >,
) {
}
