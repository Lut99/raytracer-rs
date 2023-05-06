//  SPEC.rs
//    by Lut99
// 
//  Created:
//    05 May 2023, 10:42:13
//  Last edited:
//    05 May 2023, 11:39:49
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the interfaces used for the `materials` module.
// 

use crate::math::{Colour, Ray};
use crate::specifications::objects::HitRecord;


/***** LIBRARY *****/
/// The Material trait implements any material that we can use to cover an object.
pub trait Material {
    /// Bounces (or reflects) a ray from this material.
    /// 
    /// # Arguments
    /// - `ray`: The inbound [`Ray`] that we want to scatter.
    /// - `record`: The [`HitRecord`] that determines where the hit was and what the hit normal was and such.
    /// 
    /// # Returns
    /// A tuple that represents the bounced [`Ray`] and the attenuated colour from this bounce. If [`None`] is returned for the [`Ray`], then no more bounce is necessary.
    fn scatter(&self, ray: Ray, record: HitRecord) -> (Option<Ray>, Colour);
}
