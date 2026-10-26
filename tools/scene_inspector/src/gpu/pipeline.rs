//  PIPELINE.rs
//    by Lut99
//
//  Description:
//!   Defines a render pipeline that renders one specific object and material
//!   pair.
//

use std::path::PathBuf;

use log::debug;

use super::{Buffer, Depth, Texture};


/***** CONSTANTS *****/
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914] },   // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641] },   // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];





/***** AUXILLARY *****/
/// Defines a single vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position:   [f32; 3],
    tex_coords: [f32; 2],
}

// Vertex
impl Vertex {
    /// Returns the [`VertexBufferLayout`] required to read this.
    #[inline]
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode:    wgpu::VertexStepMode::Vertex,
            attributes:   &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 }, wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}





/***** LIBRARY *****/
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    vbuffer:  Buffer<'static, &'static [Vertex]>,
    ibuffer:  Buffer<'static, &'static [u16]>,
    tex:      Texture,
}

// Constructors
impl Pipeline {
    /// Constructor for a Pipeline.
    ///
    /// # Arguments
    /// - `device`: The [`wgpu::Device`] to create the pipeline on.
    /// - `surface_format`: The format of the target rendering surface.
    /// - `tex_bind_layout`: The [`wgpu::BindGroupLayout`] of textures.
    /// - `cam_bind_layout`: The [`wgpu::BindGroupLayout`] of cameras.
    ///
    /// # Returns
    /// A new Pipeline.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        tex_sampler: &wgpu::Sampler,
        tex_bind_layout: &wgpu::BindGroupLayout,
        cam_bind_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // Prepare loading the pipeline
        debug!(target: "Pipeline::new", "Loading shaders...");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render pipeline layout"),
            bind_group_layouts: &[Some(tex_bind_layout), Some(cam_bind_layout)],
            immediate_size: 0,
        });

        // Create the pipeline
        debug!(target: "Pipeline::new", "Creating render pipeline...");
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format:     surface_format,
                    blend:      Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(Depth::depth_stencil_state()),
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview_mask: None,
            cache: None,
        });

        // Create the buffers
        debug!(target: "State::new", "Creating vertex buffer...");
        let mut vbuffer = Buffer::with_label_and_data(wgpu::BufferUsages::VERTEX, "vertex buffer", VERTICES);
        vbuffer.load_gpu(&device, |v| bytemuck::cast_slice(v));

        debug!(target: "State::new", "Creating index buffer...");
        let mut ibuffer = Buffer::with_label_and_data(wgpu::BufferUsages::INDEX, "index buffer", INDICES);
        ibuffer.load_gpu(&device, |i| bytemuck::cast_slice(i));

        // Create the texture
        debug!(target: "State::new", "Creating texture...");
        let mut tex = Texture::from_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pollo.jpg")).unwrap();
        tex.load_gpu(device, queue, tex_sampler, tex_bind_layout);

        // Done, create self
        Self { pipeline, vbuffer, ibuffer, tex }
    }
}

// GPU
impl Pipeline {
    /// Returns the internal [`wgpu::RenderPipeline`].
    pub const fn pipeline(&self) -> &wgpu::RenderPipeline { &self.pipeline }

    /// Returns the internal vertex buffer.
    pub const fn vbuf(&self) -> &Buffer<'static, &'static [Vertex]> { &self.vbuffer }

    /// Returns the internal index buffer.
    pub const fn ibuf(&self) -> &Buffer<'static, &'static [u16]> { &self.ibuffer }

    /// Returns the internal [`wgpu::BindGroup`] for the texture.
    pub const fn tex_bind_group(&self) -> &wgpu::BindGroup { self.tex.bind_group() }
}
