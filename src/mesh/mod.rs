use crate::{
    gizmo::{TransformGizmo, TransformGizmoInteraction, click_axis, drag_axis, drag_axis_end},
    gizmo_material::GizmoMaterial,
};
use bevy::{camera::visibility::{Layer, RenderLayers}, light::NotShadowCaster, prelude::*};

pub mod truncated_torus;

const GIZMO_AXIS_LENGTH: f32 = 1.3;
pub const GIZMO_RENDER_LAYER: Layer = 1;

/// Startup system that builds the procedural mesh and materials of the gizmo.
pub fn spawn_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GizmoMaterial>>,
) {
    // Define gizmo size
    let arc_radius = 1.;
    let plane_size = GIZMO_AXIS_LENGTH * 0.25;
    let plane_offset = plane_size / 2. + GIZMO_AXIS_LENGTH * 0.2;

    // Define gizmo meshes
    let arrow_tail_mesh = meshes.add(Capsule3d {
        radius: 0.04,
        half_length: GIZMO_AXIS_LENGTH * 0.5f32,
    });
    let cone_mesh = meshes.add(Cone {
        height: 0.25,
        radius: 0.10,
    });
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(plane_size, plane_size));
    let sphere_mesh = meshes.add(Sphere { radius: 0.2 });
    let rotation_mesh = meshes.add(Mesh::from(truncated_torus::TruncatedTorus {
        radius: arc_radius,
        ring_radius: 0.04,
        ..Default::default()
    }));

    // Define gizmo materials
    let (s, l) = (0.8, 0.6);
    let gizmo_matl_x = materials.add(GizmoMaterial::from(Color::hsl(0.0, s, l)));
    let gizmo_matl_y = materials.add(GizmoMaterial::from(Color::hsl(120.0, s, l)));
    let gizmo_matl_z = materials.add(GizmoMaterial::from(Color::hsl(240.0, s, l)));
    let gizmo_matl_x_sel = materials.add(GizmoMaterial::from(Color::hsl(0.0, s, l)));
    let gizmo_matl_y_sel = materials.add(GizmoMaterial::from(Color::hsl(120.0, s, l)));
    let gizmo_matl_z_sel = materials.add(GizmoMaterial::from(Color::hsl(240.0, s, l)));
    let gizmo_matl_v_sel = materials.add(GizmoMaterial::from(Color::hsl(0., 0.0, l)));

    // Build the gizmo using the variables above.
    commands
        .spawn((TransformGizmo::default(), Transform::default(), Visibility::Visible))
        .with_children(|parent| {
            // Translation Axes
            parent
                .spawn((
                    Mesh3d(arrow_tail_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_x.clone()),
                    Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_z(std::f32::consts::PI / 2.0),
                        Vec3::new(GIZMO_AXIS_LENGTH / 2.0, 0.0, 0.0),
                    )),
                    TransformGizmoInteraction::TranslateAxis {
                        original: Vec3::X,
                        axis: Vec3::X,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);
            parent
                .spawn((
                    Mesh3d(arrow_tail_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_y.clone()),
                    Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_y(std::f32::consts::PI / 2.0),
                        Vec3::new(0.0, GIZMO_AXIS_LENGTH / 2.0, 0.0),
                    )),
                    TransformGizmoInteraction::TranslateAxis {
                        original: Vec3::Y,
                        axis: Vec3::Y,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);
            parent
                .spawn((
                    Mesh3d(arrow_tail_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_z.clone()),
                    Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                        Vec3::new(0.0, 0.0, GIZMO_AXIS_LENGTH / 2.0),
                    )),
                    TransformGizmoInteraction::TranslateAxis {
                        original: Vec3::Z,
                        axis: Vec3::Z,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);

            // Translation Handles
            parent
                .spawn((
                    Mesh3d(cone_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_x_sel.clone()),
                    Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                        Vec3::new(GIZMO_AXIS_LENGTH, 0.0, 0.0),
                    )),
                    TransformGizmoInteraction::TranslateAxis {
                        original: Vec3::X,
                        axis: Vec3::X,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);
            parent
                .spawn((
                    Mesh3d(plane_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_x_sel.clone()),
                    Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                        Vec3::new(0., plane_offset, plane_offset),
                    )),
                    TransformGizmoInteraction::TranslatePlane {
                        original: Vec3::X,
                        normal: Vec3::X,
                    },
                    //NoBackfaceCulling,
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);
                //.observe(click_plane)
                //.observe(drag_plane);
            parent
                .spawn((
                    Mesh3d(cone_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_y_sel.clone()),
                    Transform::from_translation(Vec3::new(0.0, GIZMO_AXIS_LENGTH, 0.0)),
                    TransformGizmoInteraction::TranslateAxis {
                        original: Vec3::Y,
                        axis: Vec3::Y,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);
            parent
                .spawn((
                    Mesh3d(plane_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_y_sel.clone()),
                    Transform::from_translation(Vec3::new(plane_offset, 0.0, plane_offset)),
                    TransformGizmoInteraction::TranslatePlane {
                        original: Vec3::Y,
                        normal: Vec3::Y,
                    },
                    //NoBackfaceCulling,
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);
                //.observe(click_plane)
                //.observe(drag_plane);
            parent
                .spawn((
                    Mesh3d(cone_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_z_sel.clone()),
                    Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                        Vec3::new(0.0, 0.0, GIZMO_AXIS_LENGTH),
                    )),
                    TransformGizmoInteraction::TranslateAxis {
                        original: Vec3::Z,
                        axis: Vec3::Z,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);
            parent
                .spawn((
                    Mesh3d(plane_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_z_sel.clone()),
                    Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                        Vec3::new(plane_offset, plane_offset, 0.0),
                    )),
                    TransformGizmoInteraction::TranslatePlane {
                        original: Vec3::Z,
                        normal: Vec3::Z,
                    },
                    //NoBackfaceCulling,
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                //.observe(click_plane)
                //.observe(drag_plane);
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);

            // screen space drag sphere
            parent
                .spawn((
                    Mesh3d(sphere_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_v_sel.clone()),
                    TransformGizmoInteraction::TranslatePlane {
                        original: Vec3::ZERO,
                        normal: Vec3::Z,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                //.observe(click_plane)
                //.observe(drag_plane);
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);

            // Rotation Arcs
            parent
                .spawn((
                    Mesh3d(rotation_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_x.clone()),
                    Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, f32::to_radians(90.0))),
                    TransformGizmoInteraction::RotateAxis {
                        original: Vec3::X,
                        axis: Vec3::X,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                //.observe(click_rotate)
                //.observe(drag_rotate);
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);
            parent
                .spawn((
                    Mesh3d(rotation_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_y.clone()),
                    TransformGizmoInteraction::RotateAxis {
                        original: Vec3::Y,
                        axis: Vec3::Y,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                //.observe(click_rotate)
                //.observe(drag_rotate);
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);
            parent
                .spawn((
                    Mesh3d(rotation_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_z.clone()),
                    Transform::from_rotation(
                        Quat::from_axis_angle(Vec3::Z, f32::to_radians(90.0))
                            * Quat::from_axis_angle(Vec3::X, f32::to_radians(90.0)),
                    ),
                    TransformGizmoInteraction::RotateAxis {
                        original: Vec3::Z,
                        axis: Vec3::Z,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                //.observe(click_rotate)
                //.observe(drag_rotate);
                .observe(click_axis)
                .observe(drag_axis_end)
                .observe(drag_axis);
        });
}
