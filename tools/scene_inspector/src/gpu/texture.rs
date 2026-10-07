//  TEXTURE.rs
//    by Lut99
//
//  Description:
//!   Defines a convenient wrapper around a texture and its related resources.
//

use std::fs::File;
use std::io::{BufRead, BufReader, Read as _, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use image::{ImageFormat, RgbaImage};
use log::{debug, info};
use thiserror::Error;


/***** ERRORS *****/
/// Defines errors for the [`Texture`].
#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to open file {path:?}")]
    FileOpen { path: PathBuf, source: std::io::Error },
    #[error("Failed to read file {path:?}")]
    FileRead { path: PathBuf, source: std::io::Error },
    #[error("Failed to seek in file {path:?}")]
    FileSeek { path: PathBuf, source: std::io::Error },
    #[error("Failed to guess image format of {what:?}")]
    ImageGuessFormat { what: String, source: image::ImageError },
    #[error("Failed to load {what:?} as a {format:?} image")]
    ImageLoad { what: String, format: ImageFormat, source: image::ImageError },
}





/***** LIBRARY *****/
/// Represents a single texture, CPU- and GPU resources combined.
#[derive(Debug)]
pub struct Texture {
    // CPU
    /// Debug description of the resource.
    what: String,
    /// The CPU resource.
    data: RgbaImage,

    // GPU
    /// The bind group used to render the texture.
    gpu: Option<wgpu::BindGroup>,
}

// Constructors
impl Texture {
    /// Loads the texture from a file, guessing the format.
    ///
    /// # Arguments
    /// - `path`: The file to load the texture from.
    ///
    /// # Returns
    /// A new Texture that has the CPU resources loaded in.
    #[inline]
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        fn _from_path(path: &Path) -> Result<Texture, Error> {
            // Open the file
            let path: &Path = path.as_ref();
            debug!(target: "Texture::new", "Guessing format from file {path:?}...");
            let mut handle = File::open(path).map_err(|source| Error::FileOpen { path: path.into(), source })?;

            // Read enough bytes and guess the format
            let mut buf: [u8; 512] = [0; 512];
            let buf_len: usize = handle.read(&mut buf).map_err(|source| Error::FileRead { path: path.into(), source })?;
            let format: ImageFormat =
                image::guess_format(&buf[..buf_len]).map_err(|source| Error::ImageGuessFormat { what: path.display().to_string(), source })?;

            // Reset the file and load it
            handle.seek(SeekFrom::Start(0)).map_err(|source| Error::FileSeek { path: path.into(), source })?;
            Texture::from_reader_with_format(path.display().to_string(), format, BufReader::new(handle))
        }
        _from_path(path.as_ref())
    }

    /// Loads the texture from a file as the given format.
    ///
    /// # Arguments
    /// - `path`: The file to load the texture from.
    /// - `format`: The [`ImageFormat`] describing how to load the texture.
    ///
    /// # Returns
    /// A new Texture that has the CPU resources loaded in.
    #[inline]
    pub fn from_path_with_format(path: impl AsRef<Path>, format: ImageFormat) -> Result<Self, Error> {
        fn _from_path_with_format(path: &Path, format: ImageFormat) -> Result<Texture, Error> {
            // Open the file and load it as a reader
            let path: &Path = path.as_ref();
            debug!(target: "Texture::new", "Opening file {path:?}...");
            let handle = File::open(path).map_err(|source| Error::FileOpen { path: path.into(), source })?;
            Texture::from_reader_with_format(path.display().to_string(), format, BufReader::new(handle))
        }
        _from_path_with_format(path.as_ref(), format)
    }

    /// Loads the texture from a file with an apriori specified format.
    ///
    /// # Arguments
    /// - `what`: Some (debug) description of what we're loading.
    /// - `format`: The [`ImageFormat`] describing how to load the texture.
    /// - `reader`: The [`Read`] to load the texture from.
    ///
    /// # Returns
    /// A new Texture that has the CPU resources loaded in.
    #[inline]
    pub fn from_reader_with_format(what: impl Into<String>, format: ImageFormat, reader: impl BufRead + Seek) -> Result<Self, Error> {
        fn _from_reader_with_format(what: String, format: ImageFormat, reader: impl BufRead + Seek) -> Result<Texture, Error> {
            // Read the file with image
            debug!(target: "Texture::new", "Loading {what:?} as {format:?}...");
            let image = image::load(reader, format).map_err(|source| Error::ImageLoad { what: what.clone(), format, source })?;
            let data = image.into_rgba8();
            info!(target: "Texture::new", "Loaded {what:?}: {}x{}, {} byte(s)", data.dimensions().0, data.dimensions().1, 4 * data.len());

            // Store self
            Ok(Texture { what, data, gpu: None })
        }
        _from_reader_with_format(what.into(), format, reader)
    }
}

// Texture
impl Texture {
    /// Loads the GPU resources for this texture.
    ///
    /// # Arguments
    /// - `device`: A [`wgpu::Device`] to load the texture to.
    /// - `queue`: A [`wgpu::Queue`] of the `device` to run the texture load command with.
    /// - `sampler`: A [`wgpu::Sampler`] to sample the texture with.
    /// - `bind_layout`: A [`wgpu::BindGroupLayout`] to define how to bind the texture.
    ///
    /// # Errors
    /// This function can error if the GPU errors.
    pub fn load_gpu(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sampler: &wgpu::Sampler,
        bind_layout: &wgpu::BindGroupLayout,
    ) -> Result<(), Error> {
        let im_dims = self.data.dimensions();

        // Put it in a texture
        debug!(target: "State::new", "Loading texture to GPU...");
        let tex_size = wgpu::Extent3d { width: im_dims.0, height: im_dims.1, depth_or_array_layers: 1 };
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("texture {:?}", self.what)),
            size: tex_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &self.data,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * im_dims.0), rows_per_image: Some(im_dims.1) },
            tex_size,
        );

        // Define the view & bind group
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some(&format!("bind group {:?}", self.what)),
            layout:  &bind_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) }, wgpu::BindGroupEntry {
                binding:  1,
                resource: wgpu::BindingResource::Sampler(sampler),
            }],
        });

        // Set it internally
        self.gpu = Some(bind_group);

        // Done
        Ok(())
    }

    /// Frees the GPU resources for this texture.
    pub fn free_gpu(&mut self) { self.gpu = None; }



    /// Provides access to the loaded bind group.
    ///
    /// # Returns
    /// A reference to the internal [`wgpu::BindGroup`].
    ///
    /// # Panics
    /// This function panics if you didn't call [`Texture::load_gpu()`] on this texture.
    #[inline]
    #[track_caller]
    pub const fn bind_group(&self) -> &wgpu::BindGroup {
        match &self.gpu {
            Some(bg) => bg,
            None => panic!("Cannot call Texture::bind_group() before calling Texture::load_gpu()"),
        }
    }
}
