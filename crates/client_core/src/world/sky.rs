use bevy::prelude::*;
use bevy::shader::ShaderRef;
use bevy::render::render_resource::AsBindGroup;
use bevy::reflect::TypePath;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct StarrySkyMaterial {
    #[uniform(0)]
    pub sky_color: LinearRgba,
}

impl Material for StarrySkyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/starry_sky.wgsl".into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        // Disable backface culling since we are inside the sphere
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(Component)]
pub struct StarrySky;

pub fn setup_starry_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StarrySkyMaterial>>,
) {
    commands.spawn((
        (Mesh3d(meshes.add(Sphere::new(500.0).mesh().ico(5).unwrap())), MeshMaterial3d(materials.add(StarrySkyMaterial {
                sky_color: LinearRgba::new(0.01, 0.01, 0.02, 1.0),
            }))),
        StarrySky,
        bevy::light::NotShadowCaster,
        bevy::light::NotShadowReceiver,
    ));
}

pub fn starry_sky_follow_system(
    camera_query: Query<&Transform, (With<Camera3d>, Without<StarrySky>)>,
    mut sky_query: Query<&mut Transform, With<StarrySky>>,
) {
    let Ok(cam_transform) = camera_query.single() else { return; };
    for mut sky_transform in sky_query.iter_mut() {
        sky_transform.translation = cam_transform.translation;
    }
}
