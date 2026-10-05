//  BUFFER.rs
//    by Lut99
//
//  Description:
//!   Generalizes GPU buffer usage into the [`Buffer`] type.
//

use std::borrow::Cow;
use std::ops::RangeBounds;

use bytemuck::NoUninit;
use wgpu::util::DeviceExt;


/***** LIBRARY *****/
/// Represents a buffer of data (i.e., a [`Vec`]) with a GPU-part.
pub struct Buffer<T> {
    /// The CPU data to load.
    data: Vec<T>,
    /// The GPU resources to match it.
    gpu:  Option<wgpu::Buffer>,
}

// Constructors
impl<T> Default for Buffer<T> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<T> Buffer<T> {
    /// Creates a new, empty buffer without any contents.
    ///
    /// # Returns
    /// An empty, uninitialized Buffer.
    #[inline]
    pub const fn new() -> Self { Self { data: Vec::new(), gpu: None } }

    /// Creates a new, empty buffer without any contents, but with CPU memory pre-allocated for a
    /// given amount of elements.
    ///
    /// # Arguments
    /// - `capacity`: The number of elements the buffer should have _at least_ space for. See
    ///   [`Vec::with_capacity()`] for more details.
    ///
    /// # Returns
    /// An empty but initialized Buffer.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self { data: Vec::with_capacity(capacity), gpu: None } }
}

// Mutating collection
impl<T> Buffer<T> {
    /// Adds a given element to the Buffer.
    ///
    /// Note that, if you've already [created GPU resources](Buffer::load_gpu()), these still
    /// contain the buffer **without** the new element. Call [`Buffer::sync_gpu()`] to update them.
    ///
    /// # Arguments
    /// - `elem`: The new elemen`T` to add to the Buffer.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn push(&mut self, elem: T) -> &mut Self {
        self.data.push(elem);
        self
    }

    /// Allows pushing the contents of an iterator to the Buffer.
    ///
    /// Note that, if you've already [created GPU resources](Buffer::load_gpu()), these still
    /// contain the buffer **without** the new element. Call [`Buffer::sync_gpu()`] to update them.
    ///
    /// # Arguments
    /// - `iter`: The [`Iterator`] to consume and add the elements it yields to the data of the
    ///   buffer.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) -> &mut Self {
        self.data.extend(iter);
        self
    }
}
impl<T> Extend<T> for Buffer<T> {
    /// Akin to [`Buffer::extend()`] but with trait compatability.
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) { self.data.extend(iter) }
}

// Read-only collection
impl<T> Buffer<T> {
    /// Returns the number of elements in this buffer.
    #[inline]
    pub const fn len(&self) -> usize { self.data.len() }

    /// Returns whether there are any elements (false) or not (true) in this buffer.
    #[inline]
    pub const fn is_empty(&self) -> bool { self.data.is_empty() }
}

// Iterators
impl<T> Buffer<T> {
    /// Returns an iterator yielding read-only references to all elements.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> { self.data.iter() }

    /// Returns an iterator yielding mutable references to all elements.
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> { self.data.iter_mut() }
}
impl<'a, T> IntoIterator for &'a Buffer<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}
impl<'a, T> IntoIterator for &'a mut Buffer<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}
impl<T> IntoIterator for Buffer<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.data.into_iter() }
}

// GPU
impl<T> Buffer<T> {
    /// Loads the contents of the Buffer into a GPU [`wgpu::Buffer`] by manually specifying how.
    ///
    /// Note that this re-allocates the buffer entirely.
    ///
    /// # Arguments
    /// - `device`: The [`wgpu::Device`] to allocate the GPU buffer on.
    /// - `usage`: A [`wgpu::BufferUsages`] describing how this buffer will be used in the pipeline.
    /// - `label`: An (optional) label to describe the buffer in the logs.
    /// - `cast_with`: An [`FnOnce`] closure describing how to read the slice as a sequence of
    ///   bytes.
    ///
    /// # Returns
    /// Self for chaining.
    pub fn load_gpu(
        &mut self,
        device: &wgpu::Device,
        usage: wgpu::BufferUsages,
        label: Option<&str>,
        cast_with: impl FnOnce(&[T]) -> Cow<[u8]>,
    ) -> &mut Self {
        let data = cast_with(&self.data);
        self.gpu = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label, contents: &data, usage }));
        self
    }
}
impl<T: NoUninit> Buffer<T> {
    /// Loads the contents of the Buffer into a GPU [`wgpu::Buffer`] by casting a slice of `T`s
    /// into a byteslice.
    ///
    /// Note that this re-allocates the buffer entirely.
    ///
    /// # Arguments
    /// - `device`: The [`wgpu::Device`] to allocate the GPU buffer on.
    /// - `usage`: A [`wgpu::BufferUsages`] describing how this buffer will be used in the pipeline.
    /// - `label`: An (optional) label to describe the buffer in the logs.
    ///
    /// # Returns
    /// Self for chaining.
    pub fn load_gpu_directly(&mut self, device: &wgpu::Device, usage: wgpu::BufferUsages, label: Option<&str>) -> &mut Self {
        self.gpu = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label, contents: bytemuck::cast_slice(&self.data), usage }));
        self
    }
}
impl<T> Buffer<T> {
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

// Conversion
impl<const LEN: usize, T> From<[T; LEN]> for Buffer<T> {
    #[inline]
    fn from(value: [T; LEN]) -> Self { Self { data: value.into(), gpu: None } }
}
impl<T> From<Vec<T>> for Buffer<T> {
    #[inline]
    fn from(value: Vec<T>) -> Self { Self { data: value, gpu: None } }
}
impl<T> FromIterator<T> for Buffer<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self { Self { data: iter.into_iter().collect(), gpu: None } }
}
