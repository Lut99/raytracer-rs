//  UTILS.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 19:05:14
//  Last edited:
//    01 May 2023, 19:15:30
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines a few utilities used across objects.
// 

use crate::math::{AABB, Vec3};
use crate::math::aabb::surround;

use super::spec::BoundingBoxable;


/***** LIBRARY *****/
/// Helper function that computes a bounding box perfectly fitting the given iterator over [`BoundingBoxable`] objects.
/// 
/// # Arguments
/// - `objs`: The iterator that generates objects to compute the [`AABB`] of.
/// 
/// # Returns
/// The Axis-Aligned Bounding Box ([`AABB`]) that perfect fits all of the objects.
pub fn surround_list(objs: impl IntoIterator<Item = impl BoundingBoxable>) -> AABB {
    let mut objs = objs.into_iter();

    // Attempt to get the first item for the first box
    let mut aabb: AABB = match objs.next() {
        Some(obj) => obj.aabb(),
        None      => { return AABB::new(Vec3::zeroes(), Vec3::zeroes()); },
    };

    // Add any other object in the iterator
    for obj in objs {
        aabb = surround(aabb, obj.aabb());
    }

    // Done
    aabb
}
