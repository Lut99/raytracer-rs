//  STATE.rs
//    by Lut99
//
//  Description:
//!   Defines the application state.
//

use std::sync::Arc;
use std::time::Instant;

use log::{debug, info};
use thiserror::Error;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

use super::gpu::Texture;
use crate::gpu::{Depth, Pipeline};
use crate::logic::camera_controller::CameraController;
use crate::math::{Camera, Interval, Vec3};


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
    #[error("Failed to load texture")]
    TextureLoad(#[source] super::gpu::texture::Error),
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





/***** LIBRARY *****/
/// Defines the state of the [`App`](super::App).
pub struct State {
    // Windows
    /// The window to render in.
    window: Arc<Window>,

    // Render structures
    /// A configuration used to create the surface.
    config: wgpu::SurfaceConfiguration,
    /// The device we use to render.
    device: wgpu::Device,
    /// The queue that we send commands to the device.
    queue: wgpu::Queue,
    /// The surface that WGPU draws to.
    surface: wgpu::Surface<'static>,
    /// Depth buffer for correctly ordering objects.
    depth_buffer: Depth,
    /// The camera.
    cam: Camera,
    /// A controller for the camera.
    cam_controller: CameraController,

    // Pipelines
    pipeline: Pipeline,

    // State
    /// Did we configure the surface yet?
    is_surface_configured: bool,
    /// Last time an update occurred.
    last_update: Instant,
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
        debug!(target: "State::new", "Input window size: {}x{}", size.width, size.height);

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

        // Load texture related resources
        let tex_sampler = Texture::sampler(&device);
        let tex_bind_layout = Texture::bind_group_layout(&device);

        // Load the depth buffer
        let mut depth_buffer = Depth::new((config.width, config.height));
        depth_buffer.load_gpu(&device);

        // Load the camera
        debug!(target: "State::new", "Creating new Camera...");
        let mut cam = Camera::new(
            45.0,
            config.width as f64 / config.height as f64,
            Interval::new(0.1, 100.0),
            Vec3::new(0.0, 1.0, 2.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        cam.load_gpu(&device);

        // Create all pipeline combinations
        let pipeline = Pipeline::new(&device, &queue, config.format, &tex_sampler, &tex_bind_layout, cam.bind_group_layout());

        // Finally create self
        info!(target: "State::new", "Initialization success");
        Ok(Self {
            window,
            config,
            device,
            queue,
            surface,
            depth_buffer,
            cam,
            cam_controller: Default::default(),
            pipeline,
            is_surface_configured: false,
            last_update: Instant::now(),
        })
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
    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {
                // Let other systems handle it
                self.cam_controller.handle_key(code, is_pressed);
                self.update();
            },
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
        self.depth_buffer = Depth::new((width, height));
        self.depth_buffer.load_gpu(&self.device);
        self.surface.configure(&self.device, &self.config);
        self.is_surface_configured = true;
    }

    /// Updates the state.
    pub fn update(&mut self) {
        let since_last_update = self.last_update.elapsed();
        self.last_update = Instant::now();

        self.cam_controller.update_camera(since_last_update, &mut self.cam);
        self.cam.sync_gpu(&self.queue);
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
                depth_stencil_attachment: Some(self.depth_buffer.attachment()),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            // Add the pipeline
            render_pass.set_pipeline(self.pipeline.pipeline());
            render_pass.set_bind_group(0, self.pipeline.tex_bind_group(), &[]);
            render_pass.set_bind_group(1, self.cam.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.pipeline.vbuf().buffer_slice(..));
            render_pass.set_index_buffer(self.pipeline.ibuf().buffer_slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.pipeline.ibuf().get().unwrap().len() as u32, 0, 0..1);
        }

        // Submit the encoder
        self.queue.submit([encoder.finish()]);
        self.queue.present(output);

        // Done
        Ok(())
    }
}
