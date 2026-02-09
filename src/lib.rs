use bevy::prelude::*;

pub mod mesh;
mod gizmo_material;
pub mod gizmo;

#[derive(Component)]
struct TransformGizmo;

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
struct InitialTransform {
    transform: Transform,
    rotation_offset: Vec3,
}

