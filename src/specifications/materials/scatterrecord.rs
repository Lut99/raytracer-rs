//  SCATTER RECORD.rs
//    by Lut99
//
//  Description:
//!   Defines a [`ScatterRecord`], which keeps track of information in much the
//!   same way a [`HitRecord`](super::super::objects::HitRecord) does.
//

use crate::math::{Colour, Ray};


/***** AUXILLARY *****/
/// Determines that something is either sampled with PDF's or directly scattered with the given
/// ray.
#[derive(Clone, Copy, Debug)]
pub enum RayOrPDF<P> {
    /// It's a direct ray.
    Ray(Ray),
    /// It's the PDF.
    PDF(P),
}

// RayOrPDF
impl<P> RayOrPDF<P> {
    /// Maps the internal `P`DF to something else.
    ///
    /// # Arguments
    /// - `map`: Some [`FnOnce`] closure that will do the translation.
    ///
    /// # Returns
    /// A new RayOrPDF but with a different `P`.
    #[inline]
    pub fn map_pdf<P2>(self, map: impl FnOnce(P) -> P2) -> RayOrPDF<P2> {
        match self {
            Self::Ray(r) => RayOrPDF::Ray(r),
            Self::PDF(p) => RayOrPDF::PDF(map(p)),
        }
    }
}





/***** LIBRARY *****/
/// Defines the answer to a scattering run.
#[derive(Clone, Copy, Debug)]
pub struct ScatterRecord<P> {
    /// The colour the object "sticks" to the ray.
    pub attenuation: Colour,
    /// The PDF to sample with OR the Ray that is emitted if not sampled.
    pub ray_or_pdf:  RayOrPDF<P>,
}

// Constructors
impl<P> ScatterRecord<P> {
    /// Creates a new ScatterRecord that scatters with a ray (no sampling).
    ///
    /// # Arguments
    /// - `attenuation`: The [`Colour`] to stick to the resulting ray.
    /// - `ray`: The [`Ray`] that has scattered off.
    ///
    /// # Returns
    /// A new ScatterRecord with [`ScatterRecord::ray_or_pdf`] set to [`RayOrPDF::Ray`].
    #[inline]
    pub const fn from_ray(attenuation: Colour, ray: Ray) -> Self { Self { attenuation, ray_or_pdf: RayOrPDF::Ray(ray) } }

    /// Creates a new ScatterRecord that won't scatter but defers it to a PDF.
    ///
    /// # Arguments
    /// - `attenuation`: The [`Colour`] to stick to the resulting ray.
    /// - `pdf`: The `P`DF to sample from.
    ///
    /// # Returns
    /// A new ScatterRecord with [`ScatterRecord::ray_or_pdf`] set to [`RayOrPDF::PDF`].
    #[inline]
    pub const fn from_pdf(attenuation: Colour, pdf: P) -> Self { Self { attenuation, ray_or_pdf: RayOrPDF::PDF(pdf) } }
}

// RayOrPDF
impl<P> ScatterRecord<P> {
    /// Maps the internal `P`DF to something else.
    ///
    /// # Arguments
    /// - `map`: Some [`FnOnce`] closure that will do the translation.
    ///
    /// # Returns
    /// A new ScatterRecord but with a different `P`.
    #[inline]
    pub fn map_pdf<P2>(self, map: impl FnOnce(P) -> P2) -> ScatterRecord<P2> {
        ScatterRecord { attenuation: self.attenuation, ray_or_pdf: self.ray_or_pdf.map_pdf(map) }
    }
}
