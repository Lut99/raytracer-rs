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

use super::spec::BoundingBoxable;
use crate::math::aabb::surround;
use crate::math::{AABB, Vec3};


/***** LIBRARY *****/
/// Helper function that computes a bounding box perfectly fitting the given iterator over [`BoundingBoxable`] objects.
///
/// # Arguments
/// - `at`: A time at which to compute the AABB, in microseconds since the start of the scene.
/// - `objs`: The iterator that generates objects to compute the [`AABB`] of.
///
/// # Returns
/// The Axis-Aligned Bounding Box ([`AABB`]) that perfect fits all of the objects.
pub fn surround_list(t_us: u64, objs: impl IntoIterator<Item = impl BoundingBoxable>) -> AABB {
    let mut objs = objs.into_iter();

    // Attempt to get the first item for the first box
    let mut aabb: AABB = match objs.next() {
        Some(obj) => obj.aabb(t_us),
        None => {
            return AABB::new(Vec3::zeroes(), Vec3::zeroes());
        },
    };

    // Add any other object in the iterator
    for obj in objs {
        aabb = surround(aabb, obj.aabb(t_us));
    }

    // Done
    aabb
}
