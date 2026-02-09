use bevy::prelude::*;

use crate::gizmo_material::GizmoMaterial;

pub struct TransformGizmoPlugin;

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, crate::mesh::spawn_gizmo)
            .add_plugins(MaterialPlugin::<GizmoMaterial>::default());
    }
}
