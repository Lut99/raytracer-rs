//  FRAME.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 14:40:55
//  Last edited:
//    01 May 2023, 19:35:02
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the functionality to render a single frame based on a
//!   [`SceneFile`].
// 

use log::info;

use crate::math::colour::Colour;
use crate::math::vec3::{Vec3, Vector as _};
use crate::math::ray::Ray;
use crate::math::camera::Camera;
use crate::specifications::objects::{Hittable, Sphere};
use crate::hitlist::{HitItem, HitList};

use super::image::Image;
use super::generator::RayGenerator;


/***** HELPER FUNCTIONS *****/
/// Computes an Rgba quadruplet based on what the Ray hits.
/// 
/// # Arguments
/// - `ray`: The [`Ray`] who's colour to compute.
/// - `list`: A [`HitList`] that describes what to render.
/// 
/// # Returns
/// A new [`Rgba`] struct that contains the matched colour.
fn ray_colour(ray: Ray, list: &HitList) -> Colour {
    // If it hits the sphere, return the sphere colour
    let mut spheres: std::slice::Iter<HitItem<Sphere>> = list.spheres().into_iter();
    while let Some(sphere) = spheres.next() {
        // Match whether it is an object or a group
        match sphere {
            HitItem::Object(s) => {
                // Do the initial hit on the AABB
                if s.aabb.hit(ray, 0.0, f64::INFINITY) {
                    // Then hit the sphere
                    if let Some(record) = s.obj.hit(ray) {
                        // We return the colour as a gradient over the normal
                        return 0.5 * Colour::new(record.normal.x + 1.0, record.normal.y + 1.0, record.normal.z + 1.0, 1.0);
                    }
                }
            },

            HitItem::Group(g) => {
                // Skip all items in this group if we are never hitting them anyway
                if !g.aabb.hit(ray, 0.0, f64::INFINITY) {
                    for _ in 0..g.obj { spheres.next(); }
                }

                // Continue now with either the first of the group or first after the group
                continue;
            },
        }
    }

    // Otherwise, compute the background colour based on the Ray's direction; essentially, the higher the Y, the more blue
    let udir: Vec3 = ray.direct.unit();
    let t: f64 = 0.5 * (udir.y + 1.0);
    ((1.0 - t) * Colour::new(1.0, 1.0, 1.0, 0.0) + t * Colour::new(0.5, 0.7, 1.0, 0.0)).opaque()
}





/***** LIBRARY *****/
/// Implements the main rendering functionality.
/// 
/// # Arguments
/// - `image`: The [`Image`] to which we will render the scene.
/// - `list`: A [`HitList`] that describes what to render.
/// 
/// # Returns
/// A newly rendered image based on the given scene file.
pub fn render(image: &mut Image, list: &HitList) {
    info!("Rendering scene...");

    // Let us define the camera (static, for now)
    let camera: Camera = Camera::new(((image.width() as f64 / image.height() as f64) * 2.0, 2.0), 1.0);

    // Let us fire all the rays (we go top-to-bottom)
    for ((x, y), ray) in RayGenerator::new(image.dims(), camera).coords() {
        // Compute the colour of the Ray
        let colour : Colour = ray_colour(ray, list);

        // Write the colour to the image
        image[(x, y)] = colour.into();
    }

    // Done
}
