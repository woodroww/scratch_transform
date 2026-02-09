use bevy::{
    asset::uuid_handle, mesh::MeshVertexBufferLayoutRef, pbr::{MaterialPipeline, MaterialPipelineKey}, prelude::*, reflect::TypePath, render::render_resource::{
            AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
        }, shader::ShaderRef
};

const GIZMO_SHADER_HANDLE: Handle<Shader> = uuid_handle!("BB2F9219-6C9B-4D1A-A152-B300F7F9B165");

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GizmoMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}

impl From<Color> for GizmoMaterial {
    fn from(color: Color) -> Self {
        GizmoMaterial {
            color: color.into(),
        }
    }
}

impl Material for GizmoMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/gizmo_material.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/gizmo_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
