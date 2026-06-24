//  ITERATOR.rs
//    by Lut99
//
//  Description:
//!   Defines iterators producing [`Ray`]s to bounce through a scene.
//

use crate::math::{Camera, Ray};


/***** NAMESPACES *****/
/// Any traits useful to have in scope when using this module.
pub mod prelude {
    pub use super::{Castable, Distributable, Samplable};
}





/***** INTERFACES *****/
/// Wraps iterators that are [`Samples`]able.
pub trait Samplable: ExactSizeIterator<Item = (f64, f64)> {
    /// Wraps this iterator in a [`Samples`] in order to spice every coordinate with random
    /// deviations a specific number of times per coordinate.
    ///
    /// # Arguments
    /// - `n`: The number of samples to take per coordinate. Using `1` effectively disables
    ///   sampling.
    ///
    /// # Returns
    /// A new [`Samples`] with your iterator wrapped.
    fn sample(self, n: usize) -> Samples<Self>
    where
        Self: Sized;
}
impl<T: ExactSizeIterator<Item = (f64, f64)>> Samplable for T {
    #[inline]
    fn sample(self, n: usize) -> Samples<Self> { Samples::new(n, self) }
}



/// Wraps iterators that are [`Cast`]able.
pub trait Castable: ExactSizeIterator<Item = (f64, f64)> {
    /// Wraps this iterator in a [`Cast`] in order to turn every coordinate into a ray through the
    /// given [`Camera`].
    ///
    /// # Arguments
    /// - `cam`: A [`Camera`] to shoot rays through.
    /// - `dims`: The dimensions of the output image, as width x height.
    ///
    /// # Returns
    /// A new [`Cast`] with your iterator wrapped.
    fn cast(self, cam: Camera, dims: (u32, u32)) -> Cast<Self>
    where
        Self: Sized;
}
impl<T: ExactSizeIterator<Item = (f64, f64)>> Castable for T {
    #[inline]
    fn cast(self, cam: Camera, dims: (u32, u32)) -> Cast<Self> { Cast::new(cam, dims, self) }
}



/// Splits an iterator into `n` ones, each doing a chunk of the original.
pub trait Distributable: Clone + ExactSizeIterator {
    /// Wraps this iterator in a [`Split`] in order to split it into new iterators, each doing an
    /// equal chunk.
    ///
    /// The result is not _completely_ equal. If the total number of elements isn't neatly
    /// divisible by `n`, then the remainder is put in the last chunk. As such, the difference is
    /// at most `n`.
    ///
    /// # Arguments
    /// - `n`: The number of iterators to split it in.
    ///
    /// # Returns
    /// A new [`Split`] with your iterator wrapped.
    fn distribute(self, n: usize) -> Distribute<Self>
    where
        Self: Sized;
}
impl<T: Clone + ExactSizeIterator> Distributable for T {
    #[inline]
    fn distribute(self, n: usize) -> Distribute<Self> { Distribute::new(n, self) }
}





/***** LIBRARY *****/
/// Iterator yielding `(x, y)` coordinates over a grid.
#[derive(Clone, Copy, Debug)]
pub struct Coords {
    /// The current index.
    index: usize,
    /// The dimensions.
    dims:  (u32, u32),
}

// Constructors
impl Coords {
    /// Constructor for a Coords.
    ///
    /// # Arguments
    /// - `dims`: The dimensions of the grid to yield coordinates for.
    ///
    /// # Returns
    /// A new Coords that will yield XY-pairs for every coordinate in the grid with the given
    /// `dims`.
    #[inline]
    pub const fn new(dims: (u32, u32)) -> Self { Self { index: 0, dims } }
}

// Iterators
impl ExactSizeIterator for Coords {
    #[inline]
    fn len(&self) -> usize { self.dims.0 as usize * self.dims.1 as usize - self.index }
}
impl Iterator for Coords {
    type Item = (f64, f64);

    // Mandatory cycling
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.dims.0 as usize * self.dims.1 as usize {
            return None;
        }
        let x: usize = self.index % self.dims.0 as usize;
        let y: usize = self.index / self.dims.0 as usize;
        self.index += 1;
        Some((x as f64, y as f64))
    }

    // Accelerated skipping
    #[inline]
    fn count(self) -> usize { self.len() }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index += n;
        self.next()
    }

    // Optimization
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len: usize = self.len();
        (len, Some(len))
    }
}



/// Adds some random salt around a single coordinate to do random sampling.
///
/// This just yields spiced-up [coordinates](Coords).
#[derive(Clone, Copy, Debug)]
pub struct Samples<I> {
    iter:      I,
    n_samples: usize,
    coord:     Option<(f64, f64)>,
    index:     usize,
}

// Constructors
impl<I> Samples<I> {
    /// Constructor for a Samples wrapping it around some `I`terator.
    ///
    /// You can also create it through the [`Samplable`]-trait.
    ///
    /// # Arguments
    /// - `n_samples`: The number of samples to generate for every coordinate. Using `1` effectively
    ///   disables sampling.
    /// - `iter`: Some [`Iterator`] yielding [`(u32, u32)`](u32)-pairs.
    ///
    /// # Returns
    /// A new Samples.
    #[inline]
    pub const fn new(n_samples: usize, iter: I) -> Self { Self { iter, n_samples, coord: None, index: 0 } }
}

// Iterators
impl<I: ExactSizeIterator<Item = (f64, f64)>> ExactSizeIterator for Samples<I> {
    #[inline]
    fn len(&self) -> usize {
        let rem: usize = if self.coord.is_some() { self.n_samples - self.index } else { 0 };
        self.n_samples * self.iter.len() + rem
    }
}
impl<I: ExactSizeIterator<Item = (f64, f64)>> Iterator for Samples<I> {
    type Item = (f64, f64);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Take a new coord if none yet
        while self.coord.is_none() || self.index >= self.n_samples {
            self.coord = Some(self.iter.next()?);
            // NOTE: Must be after the escape above!
            self.index = 0;
        }
        let mut coord: (f64, f64) = self.coord.unwrap();

        // Add some spicing
        if self.n_samples > 1 {
            coord.0 += fastrand::f64();
            coord.1 += fastrand::f64();
        }

        // Yield it
        self.index += 1;
        Some(coord)
    }

    // Accelerated skipping
    #[inline]
    fn count(self) -> usize { self.len() }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        // Compute how many elements to skip internally, tracking how many remain
        let mut skip: usize = n / self.n_samples;
        self.index += n % self.n_samples;
        skip += self.index / self.n_samples;
        self.index = self.index % self.n_samples;

        // Now get that element
        self.coord = self.iter.nth(skip);
        self.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len: usize = self.len();
        (len, Some(len))
    }
}



/// Casts a [`Ray`] through a [`Camera`].
///
/// You can either wrap this around plain [coordinates](Coords), or coordinates spiced up by
/// [samples](Samples)
#[derive(Clone, Copy, Debug)]
pub struct Cast<I> {
    /// The nested iterator yielding pairs.
    iter: I,
    /// The dimensions of the output image.
    dims: (u32, u32),
    /// The camera used to shoot through
    cam:  Camera,
}

// Constructors
impl<I> Cast<I> {
    /// Constructor for a Cast that casts a given coordinate through a camera.
    ///
    /// # Arguments
    /// - `cam`: Some [`Camera`] to cast through.
    /// - `dims`: The dimensions of the output image.
    /// - `iter`: Some [`Iterator`] yielding coordinates (or sample-coordinate pairs) to cast
    ///   through the `cam`era.
    ///
    /// # Returns
    /// A new Cast.
    #[inline]
    pub const fn new(cam: Camera, dims: (u32, u32), iter: I) -> Self { Self { iter, dims, cam } }
}

// Iterators
impl<I: ExactSizeIterator<Item = (f64, f64)>> ExactSizeIterator for Cast<I> {
    #[inline]
    fn len(&self) -> usize { self.iter.len() }
}
impl<I: ExactSizeIterator<Item = (f64, f64)>> Iterator for Cast<I> {
    type Item = Ray;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Get the next coordinate
        let (x, y): (f64, f64) = self.iter.next()?;

        // Compute the logical values of these
        let u: f64 = x / (self.dims.0 as f64 - 1.0);
        let v: f64 = y / (self.dims.1 as f64 - 1.0);

        // Compute the Ray with those and the Camera viewport
        Some(self.cam.cast(u, v))
    }
}



/// Generic iterator splitting some other iterator in equal parts.
///
/// The other iterator must be an [`ExactSizeIterator`], and it should be efficient in its
/// [`Iterator::nth()`]-implementation.
#[derive(Clone, Copy, Debug)]
pub struct Distribute<I> {
    /// The iterator to split
    iter:     I,
    /// The length of the OG iterator.
    iter_len: usize,
    /// The current times we've split it.
    index:    usize,
    /// The total times to split it.
    times:    usize,
}

/// Constructor
impl<I: ExactSizeIterator> Distribute<I> {
    /// Constructor for the Distribute.
    ///
    /// # Arguments
    /// - `times`: The number of splits to create. Choosing `1` does nothing.
    /// - `iter`: The [`Iterator`] to split.
    ///
    /// # Returns
    /// A new Distribute.
    #[inline]
    pub fn new(times: usize, iter: I) -> Self {
        let iter_len: usize = iter.len();
        Self { iter, iter_len, index: 0, times }
    }
}

/// Iterator
impl<I: Clone + ExactSizeIterator> ExactSizeIterator for Distribute<I> {
    #[inline]
    fn len(&self) -> usize { self.times - self.index }
}
impl<I: Clone + ExactSizeIterator> Iterator for Distribute<I> {
    type Item = std::iter::Take<std::iter::Skip<I>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index: usize = self.index;
        if index < self.times.saturating_sub(1) {
            // Compute the chunk size and split the iterator in the current slot
            let chunk_size: usize = self.iter_len / self.times;
            self.index += 1;
            Some(self.iter.clone().skip(index * chunk_size).take(chunk_size))
        } else if index == self.times.saturating_sub(1) {
            // Simply take the remainder
            let chunk_size: usize = self.iter_len / self.times;
            self.index += 1;
            Some(self.iter.clone().skip(index * chunk_size).take(usize::MAX))
        } else {
            None
        }
    }

    #[inline]
    fn count(self) -> usize { self.len() }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len: usize = self.len();
        (len, Some(len))
    }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;

    #[test]
    fn test_coords() {
        assert_eq!(Coords::new((0, 0)).collect::<Vec<_>>(), Vec::new());
        assert_eq!(Coords::new((0, 1)).collect::<Vec<_>>(), Vec::new());
        assert_eq!(Coords::new((1, 0)).collect::<Vec<_>>(), Vec::new());
        assert_eq!(Coords::new((1, 1)).collect::<Vec<_>>(), vec![(0.0, 0.0)]);
        assert_eq!(Coords::new((1, 2)).collect::<Vec<_>>(), vec![(0.0, 0.0), (0.0, 1.0)]);
        assert_eq!(Coords::new((2, 1)).collect::<Vec<_>>(), vec![(0.0, 0.0), (1.0, 0.0)]);
        assert_eq!(Coords::new((2, 2)).collect::<Vec<_>>(), vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)]);

        assert_eq!(Coords::new((0, 0)).count(), 0);
        assert_eq!(Coords::new((0, 1)).count(), 0);
        assert_eq!(Coords::new((1, 0)).count(), 0);
        assert_eq!(Coords::new((1, 1)).count(), 1);
        assert_eq!(Coords::new((1, 2)).count(), 2);
        assert_eq!(Coords::new((2, 1)).count(), 2);
        assert_eq!(Coords::new((2, 2)).count(), 4);
        assert_eq!(Coords::new((2, 2)).skip(1).count(), 3);
    }

    #[test]
    fn test_samples() {
        assert_eq!(Coords::new((0, 0)).sample(1000).count(), 0);
        assert_eq!(Coords::new((0, 1)).sample(1000).count(), 0);
        assert_eq!(Coords::new((1, 0)).sample(1000).count(), 0);
        assert_eq!(Coords::new((1, 1)).sample(1000).count(), 1000);
        assert_eq!(Coords::new((1, 2)).sample(1000).count(), 2000);
        assert_eq!(Coords::new((2, 1)).sample(1000).count(), 2000);
        assert_eq!(Coords::new((2, 2)).sample(1000).count(), 4000);
        assert_eq!(Coords::new((2, 2)).sample(1000).skip(1).count(), 3999);
    }

    #[test]
    fn test_cast() {
        let cam = Camera::new((1920, 1000), 90.0, 0.0, 0.0, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 1.0, 0.0));

        assert_eq!(Coords::new((0, 0)).cast(cam, (0, 0)).collect::<Vec<_>>(), Vec::new());
        assert_eq!(Coords::new((0, 1)).cast(cam, (0, 1)).collect::<Vec<_>>(), Vec::new());
        assert_eq!(Coords::new((1, 0)).cast(cam, (1, 0)).collect::<Vec<_>>(), Vec::new());
        assert_eq!(Coords::new((2, 2)).cast(cam, (2, 2)).collect::<Vec<_>>(), vec![
            Ray::new([0.0, 0.0, 0.0], [-8.0, -5.0, -1.0]),
            Ray::new([0.0, 0.0, 0.0], [8.0, -5.0, -1.0]),
            Ray::new([0.0, 0.0, 0.0], [-8.0, 5.0, -1.0]),
            Ray::new([0.0, 0.0, 0.0], [8.0, 5.0, -1.0])
        ]);

        assert_eq!(Coords::new((0, 0)).cast(cam, (0, 0)).count(), 0);
        assert_eq!(Coords::new((0, 1)).cast(cam, (0, 1)).count(), 0);
        assert_eq!(Coords::new((1, 0)).cast(cam, (1, 0)).count(), 0);
        assert_eq!(Coords::new((1, 1)).cast(cam, (1, 1)).count(), 1);
        assert_eq!(Coords::new((1, 2)).cast(cam, (1, 2)).count(), 2);
        assert_eq!(Coords::new((2, 1)).cast(cam, (2, 1)).count(), 2);
        assert_eq!(Coords::new((2, 2)).cast(cam, (2, 2)).count(), 4);
        assert_eq!(Coords::new((2, 2)).cast(cam, (2, 2)).skip(1).count(), 3);
    }
}
