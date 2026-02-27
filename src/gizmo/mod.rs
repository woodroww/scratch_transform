use bevy::{picking::backend::PointerHits, prelude::*, window::PrimaryWindow};

use crate::{
    gizmo::debug_vectors::{DebugVectors, DebugVectorsPlugin, RotateDebugVectors},
    gizmo_material::GizmoMaterial,
};

pub mod debug_vectors;

#[derive(Component)]
pub struct GizmoPickSource;

#[derive(Component, Debug, Default)]
pub struct PickSelection {
    pub is_selected: bool,
    pub initial_transform: Transform,
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
    screen_drag_start: Vec2,
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
            .add_systems(Update, check_selection)
            //.add_plugins(DebugVectorsPlugin)
            .add_plugins(MaterialPlugin::<GizmoMaterial>::default());
    }
}

fn check_selection(
    mut query: Query<(Entity, &PickSelection, &GlobalTransform), Without<TransformGizmo>>,
    mut gizmo: Query<(&mut Transform, &mut TransformGizmo)>,
) {
    //let selected: Vec<_> = query.iter().filter(|(_, p, _)| p.is_selected).collect();
    //info!("selected.len() {}", selected.len());

    let Ok((mut gizmo_transform, mut gizmo)) = gizmo.single_mut() else {
        warn!("getting main gizmo error");
        return;
    };
    if gizmo.current_interaction.is_some() {
        return;
    }

    let mut transform = Transform::default();
    let mut pick_count = 0;
    for (_entity, _pick, trans) in query.iter_mut().filter(|(_, p, _)| p.is_selected) {
        transform.translation += trans.translation();
        transform.rotation = trans.rotation();
        pick_count += 1;
    }
    transform.translation /= pick_count as f32;

    gizmo_transform.translation = transform.translation;
    gizmo_transform.rotation = transform.rotation;

    // rotation ? scale ?
    gizmo.current_interaction = None;
    gizmo.drag_start = None;
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

pub fn drag_start(
    drag: On<Pointer<DragStart>>,
    interaction_query: Query<&TransformGizmoInteraction, Without<TransformGizmo>>,
    mut gizmo: Query<(&GlobalTransform, &Transform, &mut TransformGizmo)>,
    mut hit_reader: MessageReader<PointerHits>,
    mut item_query: Query<(&Transform, &mut PickSelection), Without<TransformGizmo>>,
) {
    debug_assert_eq!(interaction_query.iter().len(), 13);

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

    //let position = drag.pointer_location.position;
    //info!("click position: {:#?}", position);
    //println!("min entity: {:?}", min_entity);
    debug_assert_eq!(min_entity, Some(drag.entity));
    //println!("min data:   {:?}", min_data);

    // if there are multiple gizmos allowed we're going to have to find the one clicked
    // but for now this
    let Ok((main_global_transform, main_transform, mut transform_gizmo)) = gizmo.single_mut()
    else {
        warn!("getting main gizmo error");
        return;
    };

    for (selected_transform, mut pick) in item_query
        .iter_mut()
        .filter(|(_, pick)| pick.is_selected)
    {
        println!("saving initial_transform");
        pick.initial_transform = *selected_transform;
    }

    let Ok(interaction) = interaction_query.get(drag.entity) else {
        warn!("transform_query couldn't find entity from click");
        return;
    };

    transform_gizmo.current_interaction = Some(*interaction);
    transform_gizmo.drag_start = Some(min_data.unwrap().position.unwrap());
    transform_gizmo.screen_drag_start = drag.pointer_location.position;
    transform_gizmo.initial_transform = *main_transform;
    transform_gizmo.initial_global_transform = *main_global_transform;
}

pub fn drag_axis(
    drag: On<Pointer<Drag>>,
    pick_cam: Query<(&Camera, &GlobalTransform), With<GizmoPickSource>>,
    windows: Query<&mut Window, With<PrimaryWindow>>,
    mut gizmo_query: Query<(&mut Transform, &GlobalTransform, &mut TransformGizmo)>,
    debug_vectors: Option<ResMut<DebugVectors>>,
    //mut rotate_debug_vectors: Option<ResMut<RotateDebugVectors>>,
    mut item_query: Query<(&mut Transform, &PickSelection), Without<TransformGizmo>>,
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

    let current_pointer = drag.pointer_location.position;
    let Some(picking_ray) =
        ray_from_screenspace(current_pointer, picking_camera, global_cam_tran, window)
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
            let normalized_translation_axis = (initial_transform.rotation * axis).normalize();
            let vertical_vector = picking_ray
                .direction
                .cross(normalized_translation_axis)
                .normalize();
            let plane_normal = normalized_translation_axis
                .cross(vertical_vector)
                .normalize();
            let plane_origin = drag_start;
            let Some(ray_plane_intersection) =
                intersect_plane(picking_ray, plane_normal, plane_origin)
            else {
                warn!("what? None cursor_plane_intersection");
                return;
            };
            let cursor_vector: Vec3 = ray_plane_intersection - plane_origin;
            let plane = InfinitePlane3d::new(normalized_translation_axis);
            let isometry = Isometry3d::from_translation(plane_origin);
            let signed_distance = plane.signed_distance(isometry, ray_plane_intersection);
            let translation = normalized_translation_axis * signed_distance;
            let new_translation = initial_transform.translation + translation;

            if let Some(mut debug) = debug_vectors {
                *debug = DebugVectors {
                    translation_axis: normalized_translation_axis,
                    vertical_vector,
                    plane_normal,
                    picking_ray,
                    plane_origin,
                    ray_plane_intersection,
                    cursor_vector,
                    signed_distance,
                };
            }

            gizmo_local_transform.translation = new_translation;

            for (mut selected_transform, pick) in
                item_query.iter_mut().filter(|(_, pick)| pick.is_selected)
            {
                println!("selected_transform");
                selected_transform.translation = pick.initial_transform.translation + translation;
            }
        }
        TransformGizmoInteraction::TranslatePlane { original, normal } => {
            // if this is the center of the gizmo screen space translator
            let plane_normal = if original == Vec3::ZERO {
                global_cam_tran.forward().as_vec3()
            } else {
                initial_transform.rotation * normal
            };
            let Some(ray_plane_intersection) =
                intersect_plane(picking_ray, plane_normal, drag_start)
            else {
                warn!("what? None cursor_plane_intersection");
                return;
            };
            let translation = ray_plane_intersection - drag_start;
            gizmo_local_transform.translation = gizmo.initial_transform.translation + translation;

            for (mut selected_transform, pick) in
                item_query.iter_mut().filter(|(_, pick)| pick.is_selected)
            {
                println!("selected_transform");
                selected_transform.translation = pick.initial_transform.translation + translation;
            }
        }
        TransformGizmoInteraction::RotateAxis { original: _, axis } => {
            let Ok(screen_gizmo_center) =
                picking_camera.world_to_viewport(global_cam_tran, initial_transform.translation)
            else {
                warn!("what no screen_pos!");
                return;
            };

            let mut start = gizmo.screen_drag_start - screen_gizmo_center;
            start.y = -start.y;
            let mut current = current_pointer - screen_gizmo_center;
            current.y = -current.y;

            let mut diff_angle = start.angle_to(current);

            // Your current logic for calculating rotation through screen-space angles is a solid
            // start, but it contains a common pitfall: flipping the rotation direction depending
            // on which "side" of the object you are looking at.
            // The primary issues in your snippet are the axis reference frame and the missing
            // view-direction check.

            // 2. Determine if the axis is pointing "away" from the camera
            // We use the dot product of the world-space axis and the camera's forward vector
            let world_axis = initial_transform.rotation * axis;
            let camera_forward = global_cam_tran.forward();

            // The Dot Product Check: This is the "secret sauce" for 3D gizmos. It ensures that
            // dragging "clockwise" on the screen always feels like clockwise rotation on the
            // object, regardless of your perspective.
            // If the axis and camera-forward point in the same general direction,
            // the user is looking at the "back" of the rotation plane.
            if world_axis.dot(*camera_forward) > 0.0 {
                diff_angle *= -1.0;
            }

            let rotation = Quat::from_axis_angle(axis, diff_angle);
            gizmo_local_transform.rotation = initial_transform.rotation * rotation;

            for (mut selected_transform, pick) in
                item_query.iter_mut().filter(|(_, pick)| pick.is_selected)
            {
                selected_transform.rotation = pick.initial_transform.rotation * rotation; 
            }
        }
        TransformGizmoInteraction::ScaleAxis { original: _, axis } => {
            /*
            let normalized_translation_axis = (initial_transform.rotation * axis).normalize();
            let vertical_vector = picking_ray
                .direction
                .cross(normalized_translation_axis)
                .normalize();
            let plane_normal = normalized_translation_axis
                .cross(vertical_vector)
                .normalize();
            let plane_origin = drag_start;
            let Some(ray_plane_intersection) =
                intersect_plane(picking_ray, plane_normal, plane_origin)
            else {
                warn!("what? None cursor_plane_intersection");
                return;
            };
            let cursor_vector: Vec3 = ray_plane_intersection - plane_origin;
            let len = cursor_vector.length();
            */
        }
    }
}

pub fn drag_end(
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
    mut gizmo: Query<&mut TransformGizmo>,
) {
    let Ok(mut gizmo) = gizmo.single_mut() else {
        warn!("getting main gizmo error");
        return;
    };

    gizmo.current_interaction = None;
    gizmo.drag_start = None;
    info!("drag_end");
}
