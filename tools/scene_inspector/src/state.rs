//  STATE.rs
//    by Lut99
//
//  Description:
//!   Defines the application state.
//

use std::sync::Arc;

use log::{debug, info};
use thiserror::Error;
use wgpu::util::DeviceExt as _;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;


/***** ERRORS *****/
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to request a wgpu adapter.
    #[error("Failed to request WGPU adapter")]
    Adapter(#[source] wgpu::RequestAdapterError),
    /// Failed to request a wgpu device.
    #[error("Failed to request WGPU device")]
    Device(#[source] wgpu::RequestDeviceError),
    /// The surface did not have any alpha modes.
    #[error("No alpha modes for requested WGPU surface")]
    NoSurfaceAlphaModes,
    /// The surface did not have any formats.
    #[error("No formats for requested WGPU surface")]
    NoSurfaceFormats,
    /// The surface did not have any present modes.
    #[error("No present modes for requested WGPU surface")]
    NoSurfacePresentModes,
    /// Failed to create the wgpu surface.
    #[error("Failed to create WGPU surface")]
    Surface(#[source] wgpu::CreateSurfaceError),
    /// Lost the active surface.
    #[error("Active surface was lost")]
    SurfaceLost,
}

// Conversion
impl From<wgpu::RequestAdapterError> for Error {
    #[inline]
    fn from(value: wgpu::RequestAdapterError) -> Self { Self::Adapter(value) }
}
impl From<wgpu::RequestDeviceError> for Error {
    #[inline]
    fn from(value: wgpu::RequestDeviceError) -> Self { Self::Device(value) }
}
impl From<wgpu::CreateSurfaceError> for Error {
    #[inline]
    fn from(value: wgpu::CreateSurfaceError) -> Self { Self::Surface(value) }
}





/***** CONSTANTS *****/
pub const VERTICES: &[Vertex] =
    &[Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] }, Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] }, Vertex {
        position: [0.5, -0.5, 0.0],
        color:    [0.0, 0.0, 1.0],
    }];





/***** AUXILLARY *****/
/// Defines a single vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color:    [f32; 3],
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
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}





/***** LIBRARY *****/
/// Defines the state of the [`App`](super::App).
pub struct State {
    // Windows
    /// The window to render in.
    window: Arc<Window>,

    // Render structures
    /// A configuration used to create the surface.
    config:   wgpu::SurfaceConfiguration,
    /// The device we use to render.
    device:   wgpu::Device,
    /// The pipeline that renders for us.
    pipeline: wgpu::RenderPipeline,
    /// The queue that we send commands to the device.
    queue:    wgpu::Queue,
    /// The surface that WGPU draws to.
    surface:  wgpu::Surface<'static>,

    // Data
    /// A buffer for storing vertices to render.
    vertex_buffer:     wgpu::Buffer,
    /// The number of vertices in the `vertex_buffer`.
    vertex_buffer_len: u32,

    // State
    /// Did we configure the surface yet?
    is_surface_configured: bool,
}

// Constructors
impl State {
    /// Creates a new State.
    ///
    /// # Arguments
    /// - `window`: A [`Window`] to build the app around.
    ///
    /// # Returns
    /// A new State.
    #[inline]
    pub async fn new(window: Arc<Window>) -> Result<Self, Error> {
        let size = window.inner_size();

        // Create a new WGPU instance
        debug!(target: "State::new", "Creating WGPU instance...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        // Use that to create a surface & adapter
        debug!(target: "State::new", "Creating WGPU surface...");
        let surface = instance.create_surface(window.clone())?;
        debug!(target: "State::new", "Requesting WGPU adapter...");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await?;

        // Create a device
        debug!(target: "State::new", "Requesting WGPU device...");
        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        let ainfo = device.adapter_info();
        debug!(target: "State::new", "Device {:?} thru {:?}", ainfo.name, ainfo.backend);

        // Create surface configurations
        debug!(target: "State::new", "Generating surface config...");
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .or_else(|| surface_caps.formats.get(0).copied())
            .ok_or(Error::NoSurfaceFormats)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes.get(0).copied().ok_or(Error::NoSurfacePresentModes)?,
            alpha_mode: surface_caps.alpha_modes.get(0).copied().ok_or(Error::NoSurfaceAlphaModes)?,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        // Prepare loading the pipeline
        debug!(target: "State::new", "Loading shaders...");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        // Create the pipeline
        debug!(target: "State::new", "Creating render pipeline...");
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
                    format:     config.format,
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview_mask: None,
            cache: None,
        });

        // Create the buffer
        debug!(target: "State::new", "Creating vertex buffer...");
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage:    wgpu::BufferUsages::VERTEX,
        });
        let vertex_buffer_len: u32 = VERTICES.len() as u32;

        // Finally create self
        info!(target: "State::new", "Initialization success");
        Ok(Self { window, config, device, pipeline, queue, surface, vertex_buffer, vertex_buffer_len, is_surface_configured: false })
    }
}

// Events
impl State {
    // Updating
    /// Handles key presses.
    ///
    /// # Arguments
    /// - `event_loop`: The [`ActiveEventLoop`] that we can use to communicate with the rest of
    ///   the app.
    /// - `code`: The pressed key's code.
    /// - `is_pressed`: Whether the key is pressed or not.
    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => return,
        }
    }

    /// Handles a resize of the internal window.
    ///
    /// # Arguments
    /// - `width`: The new window width.
    /// - `height`: The new window height.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.is_surface_configured = true;
    }

    /// Updates the state.
    pub fn update(&mut self) {
        /* TODO */
    }


    // Rendering
    /// Renders a new frame to the internal window.
    pub fn render(&mut self) -> Result<(), Error> {
        self.window.request_redraw();

        // Render only when the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        // Get the texture to render to.
        let output = match self.surface.get_current_texture() {
            // Succes states
            wgpu::CurrentSurfaceTexture::Success(tex) | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,

            // Frames we miss
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Validation => return Ok(()),

            // Frames we need to update for
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            },

            // Finally, error frames
            wgpu::CurrentSurfaceTexture::Lost => return Err(Error::SurfaceLost),
        };

        // Create a view to the surface texture
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Build the encoder that creates the command buffer for rendering
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });
        {
            // Create the render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            // Add the pipeline
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.vertex_buffer_len, 0..1);
        }

        // Submit the encoder
        self.queue.submit([encoder.finish()]);
        self.queue.present(output);

        // Done
        Ok(())
    }
}
