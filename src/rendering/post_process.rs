//! Horror post-processing effects: film grain, CRT scanlines, and vignette.
//!
//! Implements a fullscreen post-processing pass using Bevy 0.15's render graph.

use bevy::{
    asset::load_internal_asset,
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FragmentState, MultisampleState, Operations,
            PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            ShaderType, TextureFormat, TextureSampleType,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
        RenderApp,
    },
};

/// Handle to the post-process shader.
const POST_PROCESS_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x8a3d7f9e2b4c6a1d5e8f7c3b9a2d4e6f);

/// Plugin that adds horror post-processing effects.
pub struct HorrorPostProcessPlugin;

impl Plugin for HorrorPostProcessPlugin {
    fn build(&self, app: &mut App) {
        // Load the shader
        load_internal_asset!(
            app,
            POST_PROCESS_SHADER_HANDLE,
            "../../assets/shaders/post_process.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins((
            ExtractComponentPlugin::<PostProcessSettings>::default(),
            UniformComponentPlugin::<PostProcessSettings>::default(),
        ));

        // Add system to update time
        app.add_systems(Update, update_post_process_time);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(Core3d, PostProcessLabel)
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    PostProcessLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<PostProcessPipeline>();
    }
}

/// Label for the post-process render node.
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PostProcessLabel;

/// Settings for horror post-processing effects.
/// Add this component to your camera to enable effects.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct PostProcessSettings {
    /// Film grain intensity (0.0 = none, 0.15 = heavy). Default: 0.006
    pub grain_intensity: f32,
    /// Film grain animation speed (lower = slower drift). Default: 0.8
    pub grain_speed: f32,
    /// Film grain coarseness (lower = coarser pattern). Default: 180.0
    pub grain_coarseness: f32,
    /// CRT scanline intensity (0.0 = none, 0.3 = heavy). Default: 0.08
    pub scanline_intensity: f32,
    /// Number of scanlines (based on vertical resolution). Default: 320.0
    pub scanline_count: f32,
    /// Vignette darkness at corners (0.0 = none, 0.5 = heavy). Default: 0.20
    pub vignette_intensity: f32,
    /// Vignette radius (0.5 = corners only, 0.3 = more coverage). Default: 0.60
    pub vignette_radius: f32,
    /// Animation time (updated automatically)
    pub time: f32,
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            grain_intensity: 0.006,
            grain_speed: 0.8,
            grain_coarseness: 180.0,
            scanline_intensity: 0.08,
            scanline_count: 320.0,
            vignette_intensity: 0.20,
            vignette_radius: 0.60,
            time: 0.0,
        }
    }
}

impl PostProcessSettings {
    /// Create PostProcessSettings from VisualConfig.
    pub fn from_config(config: &super::visual_config::VisualConfig) -> Self {
        Self {
            grain_intensity: config.grain_intensity,
            grain_speed: config.grain_speed,
            grain_coarseness: config.grain_coarseness,
            scanline_intensity: config.scanline_intensity,
            scanline_count: config.scanline_count,
            vignette_intensity: config.vignette_intensity,
            vignette_radius: config.vignette_radius,
            time: 0.0,
        }
    }
}

/// System to update the time uniform for animated grain.
fn update_post_process_time(time: Res<Time>, mut query: Query<&mut PostProcessSettings>) {
    for mut settings in &mut query {
        settings.time = time.elapsed_secs();
    }
}

/// The render node for post-processing.
#[derive(Default)]
struct PostProcessNode;

impl ViewNode for PostProcessNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static DynamicUniformIndex<PostProcessSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<PostProcessPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<PostProcessSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "post_process_bind_group",
            &post_process_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &post_process_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

/// Resource containing the post-process pipeline.
#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "post_process_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<PostProcessSettings>(true),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("post_process_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader: POST_PROCESS_SHADER_HANDLE,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::Rgba8UnormSrgb,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: false,
                });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}
