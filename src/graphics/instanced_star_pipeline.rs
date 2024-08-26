use bevy::prelude::*;
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
    },
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{GpuBufferInfo, GpuMesh, MeshVertexBufferLayoutRef},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{*,binding_types::uniform_buffer},
        renderer::{RenderDevice,RenderQueue},
        view::ExtractedView,
        Render, RenderApp, RenderSet,
    },
};
use bytemuck::{Pod, Zeroable};

pub struct StarMaterialPlugin;

impl Plugin for StarMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            (ExtractComponentPlugin::<StarInstanceMaterialData>::default(),
            ExtractComponentPlugin::<crate::camera::CameraMain>::default())        
        );
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawStar>()
            .init_resource::<SpecializedMeshPipelines<StarPipeline>>()
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSet::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSet::PrepareResources),
                    prepare_star_uniform_bind_groups.in_set(RenderSet::PrepareBindGroups),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<StarPipeline>()
            .init_resource::<StarUniformsData>();
    }
}

#[derive(Component, Deref)]
pub struct StarInstanceMaterialData(pub Vec<StarInstanceData>);

impl ExtractComponent for StarInstanceMaterialData {
    type QueryData = &'static StarInstanceMaterialData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_,Self::QueryData>) -> Option<Self> {
        Some(StarInstanceMaterialData(item.0.clone()))
    }
}

#[derive(Clone,Copy,Pod,Zeroable)]
#[repr(C)]
pub struct StarInstanceData {
    pub position : Vec3,
    pub star_radius : f32,
    pub color: [f32; 4],
}

// A bit dense
// This is where it's decided that we're rendering in the transparent phase
// But otherwise it seems to be all boilerplate
#[allow(clippy::too_many_arguments)]
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    star_pipeline: Res<StarPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<StarPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<GpuMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<Entity, With<StarInstanceMaterialData>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    mut views: Query<(Entity, &ExtractedView)>,
) {
    let draw_star = transparent_3d_draw_functions.read().id::<DrawStar>();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

    for (view_entity, view) in &mut views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view_entity) else {
            continue;
        };

        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);

        let rangefinder = view.rangefinder3d();
        for entity in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(entity) else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let key = view_key
                | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology())
                | bevy::pbr::alpha_mode_pipeline_key(AlphaMode::Blend, &msaa);

            let pipeline = pipelines
                .specialize(&pipeline_cache, &star_pipeline, key, &mesh.layout)
                .unwrap();
            transparent_phase.add(Transparent3d {
                entity,
                pipeline,
                draw_function: draw_star,
                distance: rangefinder.distance_translation(&mesh_instance.translation),
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::NONE,
            });
        }
    }
}

#[derive(Component)]
struct InstanceBuffer {
    buffer: Buffer,
    length: usize
}

// Put the instance buffer data on the GPU
// Boiler plate, no customisatino required
fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &StarInstanceMaterialData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

#[derive(Resource)]
struct StarPipeline {
    shader : Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    uniforms_layout: BindGroupLayout
}

#[derive(Component, Default, Clone, Copy, ShaderType)]
struct StarInstancingUniforms {
    system_transition_factor: f32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    _webgl2_padding: Vec3,
}

impl FromWorld for StarPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh_pipeline = world.resource::<MeshPipeline>();
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "star_instancing_bind_group_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX | ShaderStages::FRAGMENT,                
                uniform_buffer::<StarInstancingUniforms>(true),
            ),
        );

        StarPipeline {
            shader : world.load_asset("shaders/star_instancing.wgsl"),
            mesh_pipeline: mesh_pipeline.clone(),
            uniforms_layout: layout
        }
    }
}

impl SpecializedMeshPipeline for StarPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor.layout.push(self.uniforms_layout.clone());

        descriptor.vertex.shader = self.shader.clone();
        // THIS IS WHERE THE SHADER-SIDE BUFFER LAYOUT IS DEFINED
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<StarInstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        Ok(descriptor)
    }
}

type DrawStar = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetStarUniformsBindGroup<2>,
    DrawMeshInstanced,
);

#[derive(Resource)]
struct StarUniformsData {
    uniform_buffer : UniformBuffer<StarInstancingUniforms>,
    bind_group : Option<BindGroup>
}

impl FromWorld for StarUniformsData {
    fn from_world(_world : &mut World) -> Self {
        Self {
            uniform_buffer : UniformBuffer::from(StarInstancingUniforms { system_transition_factor : 0.}),
            bind_group : None
        }
    }
}

fn prepare_star_uniform_bind_groups(
    //mut commands : Commands,
    device : Res<RenderDevice>,
    queue : Res<RenderQueue>,
    star_pipeline : Res<StarPipeline>,
    mut uniforms_data : ResMut<StarUniformsData>,
    cam_query: Query<&crate::camera::CameraMain>,
) {
    let cam = cam_query.get_single().expect("couldn't find camera!");

    uniforms_data.uniform_buffer.set(StarInstancingUniforms {
        system_transition_factor : cam.adjusted_mode_transition(),
    });
    uniforms_data.uniform_buffer.write_buffer(&device, &queue);

    let Some(uniform_buffer_binding) = uniforms_data.uniform_buffer.binding() else {return; };

    uniforms_data.bind_group = Some(device.create_bind_group(
        "star_instancing_bind_group",
        &star_pipeline.uniforms_layout,
        &BindGroupEntries::single(
            uniform_buffer_binding
        ),
    ));
}

struct SetStarUniformsBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetStarUniformsBindGroup<I> {
    type Param = SRes<StarUniformsData>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        _view: (),
        _: Option<()>,
        bindgroup: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>
    ) -> RenderCommandResult {
        let bindgroup = bindgroup.into_inner();
        let Some(bindgroup) = &bindgroup.bind_group else {
            return RenderCommandResult::Failure;
        };
        pass.set_bind_group(I, bindgroup, &[0]);
        RenderCommandResult::Success
    }
}

struct DrawMeshInstanced;
// PRETTY MUCH JUST BOILERPLATE
impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type  Param = (SRes<RenderAssets<GpuMesh>>, SRes<RenderMeshInstances>);
    type ViewQuery = ();
    type ItemQuery = Read<InstanceBuffer>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        instance_buffer: Option<&'w InstanceBuffer>,
        (meshes,render_mesh_instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>
    ) -> RenderCommandResult {
        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.entity())
        else {
            return RenderCommandResult::Failure;
        };
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Failure;
        };
        let Some(instance_buffer) = instance_buffer else {
            return RenderCommandResult::Failure;
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..),0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed => {
                pass.draw(0..gpu_mesh.vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}