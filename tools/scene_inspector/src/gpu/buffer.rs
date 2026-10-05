//  BUFFER.rs
//    by Lut99
//
//  Description:
//!   Generalizes GPU buffer usage into the [`Buffer`] type.
//

use std::ops::RangeBounds;

use wgpu::util::DeviceExt as _;


/***** LIBRARY *****/
/// Represents a buffer of data (i.e., a [`Vec`]) with a GPU-part.
pub struct Buffer<'a, T> {
    // CPU
    /// The CPU data to load.
    data: Option<T>,

    // GPU
    /// A label that represents this buffer in the debug logs, if any.
    label:  Option<&'a str>,
    /// What the data will be used for.
    usages: wgpu::BufferUsages,
    /// The GPU resources to match it.
    gpu:    Option<wgpu::Buffer>,
}

// Constructors
impl<T> Buffer<'static, T> {
    /// Creates a new, empty buffer without any contents.
    ///
    /// # Arguments
    /// - `usages`: The [`wgpu::BufferUsages`] describing how the GPU part of the buffer will be
    ///   used.
    ///
    /// # Returns
    /// An empty, uninitialized Buffer.
    #[inline]
    pub const fn new(usages: wgpu::BufferUsages) -> Self { Self { data: None, label: None, usages, gpu: None } }

    /// Creates a buffer with given contents.
    ///
    /// # Arguments
    /// - `usages`: The [`wgpu::BufferUsages`] describing how the GPU part of the buffer will be
    ///   used.
    /// - `data`: The da`T`a to store in the buffer.
    ///
    /// # Returns
    /// A buffer wrapping the given `data`.
    #[inline]
    pub const fn with_data(usages: wgpu::BufferUsages, data: T) -> Self { Self { data: Some(data), label: None, usages, gpu: None } }
}
impl<'a, T> Buffer<'a, T> {
    /// Creates a buffer with given label.
    ///
    /// # Arguments
    /// - `usages`: The [`wgpu::BufferUsages`] describing how the GPU part of the buffer will be
    ///   used.
    /// - `label`: A label describing this buffer for debugging.
    ///
    /// # Returns
    /// An empty, uninitialized Buffer.
    #[inline]
    pub const fn with_label(usages: wgpu::BufferUsages, label: &'a str) -> Self { Self { data: None, label: Some(label), usages, gpu: None } }

    /// Creates a buffer with given label and contents.
    ///
    /// # Arguments
    /// - `usages`: The [`wgpu::BufferUsages`] describing how the GPU part of the buffer will be
    ///   used.
    /// - `label`: A label describing this buffer for debugging.
    /// - `data`: The da`T`a to store in the buffer.
    ///
    /// # Returns
    /// A buffer wrapping the given `data`.
    #[inline]
    pub const fn with_label_and_data(usages: wgpu::BufferUsages, label: &'a str, data: T) -> Self {
        Self { data: Some(data), label: Some(label), usages, gpu: None }
    }
}

// Collection
impl<'a, T> Buffer<'a, T> {
    /// Writes the CPU contents of the buffer.
    ///
    /// Note that these aren't yet loaded to the GPU; call [`Buffer::load_gpu()`] to do so.
    ///
    /// # Arguments
    /// - `data`: The new da`T`a to set in the Buffer.
    ///
    /// # Returns
    /// The old data, if any.
    #[inline]
    pub const fn set(&mut self, elem: T) -> Option<T> { self.data.replace(elem) }

    /// Gets CPU contents of the buffer as read-only.
    ///
    /// # Returns
    /// The internal data, if any.
    #[inline]
    pub const fn get(&self) -> Option<&T> { self.data.as_ref() }

    /// Gets CPU contents of the buffer as mutable.
    ///
    /// Note that any changes to the object aren't yet loaded to the GPU; call
    /// [`Buffer::load_gpu()`] to do so.
    ///
    /// # Returns
    /// The internal data, if any.
    #[inline]
    pub const fn get_mut(&mut self) -> Option<&mut T> { self.data.as_mut() }

    /// Clears the CPU contents of the buffer.
    ///
    /// Note that the GPU buffer still has the old contents of the buffer.
    ///
    /// # Returns
    /// The internal data, if any.
    #[inline]
    pub const fn take(&mut self) -> Option<T> { self.data.take() }
}

// Collection metadata
impl<'a, T> Buffer<'a, T> {
    /// Returns whether there is any data (false) or not (true) in this buffer.
    #[inline]
    pub const fn is_empty(&self) -> bool { self.data.is_none() }
}

// GPU
impl<'a, T> Buffer<'a, T> {
    /// Loads the contents of the Buffer into a GPU [`wgpu::Buffer`] by casting a slice of `T`s
    /// into a byteslice.
    ///
    /// Note that this re-allocates the GPU buffer entirely.
    ///
    /// # Arguments
    /// - `device`: The [`wgpu::Device`] to allocate the GPU buffer on.
    /// - `cast`: A function that casts `&T` into a byte slice.
    pub fn load_gpu(&mut self, device: &wgpu::Device, cast: impl FnOnce(&T) -> &[u8]) {
        self.gpu = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    self.label,
            contents: self.data.as_ref().map(cast).unwrap_or(&[]),
            usage:    self.usages,
        }));
    }

    /// Returns a [`wgpu::BufferView`] that can be rendered.
    ///
    /// # Arguments
    /// - `slice`: The part of the buffer (as byte indices) to actually use.
    #[track_caller]
    pub fn buffer_slice(&self, slice: impl RangeBounds<wgpu::BufferAddress>) -> wgpu::BufferSlice<'_> {
        let Some(buffer) = &self.gpu else { panic!("Cannot call Buffer::buffer_slice() before loading GPU data") };
        buffer.slice(slice)
    }
}
