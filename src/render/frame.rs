//  FRAME.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 14:40:55
//  Last edited:
//    30 Apr 2023, 12:52:26
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the functionality to render a single frame based on a
//!   [`SceneFile`].
// 

use log::info;

use crate::math::colour::Colour;
use crate::math::vec3::{dot3, Vec3, Vector as _};
use crate::math::ray::Ray;
use crate::math::camera::Camera;
use crate::specifications::objects::Sphere;
use crate::hitlist::{HitItem, HitList};

use super::image::Image;
use super::generator::RayGenerator;


/***** HELPER FUNCTIONS *****/
/// Computes the hit point of the given ray with the given sphere, if any.
/// 
/// # Arguments
/// - `ray`: The [`Ray`] to check if it hits.
/// - `sphere`: The [`Sphere`] to compute the hit with.
/// 
/// # Returns
/// The point along the given Ray if there is a hit, or else [None].
fn hit_sphere(ray: Ray, sphere: &Sphere) -> Option<f64> {
    // Compute the distance between the origin of the ray and the center of the sphere
    let oc: Vec3 = ray.origin - sphere.center;

    // We compute `a`, `b` and `c` in the classic ABC-formula. This we do to find the intersections between the Ray (origin + t*direction) and the sphere (x^2 + y^2 + z^2 = r^2).
    // For more explanation, see the tutorial (<https://raytracing.github.io/books/RayTracingInOneWeekend.html#addingasphere/ray-sphereintersection>)
    let a      : f64 = ray.direct.length2();
    let half_b : f64 = dot3(oc, ray.direct);
    let c      : f64 = oc.length2() - sphere.radius * sphere.radius;

    // Compute the discriminant only, since we're only interested in the number of roots
    // D < 0 -> no intersection, D == 0 -> one intersection (touching side), D > 0 -> two intersections (passing through)
    let d: f64 = half_b*half_b - a*c;
    if d >= 0.0 {
        Some((-half_b - d.sqrt()) / a)
    } else {
        None
    }
}



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
                    if let Some(point) = hit_sphere(ray, &s.obj) {
                        // Compute the normal
                        let normal: Vec3 = (ray.at(point) - s.obj.center).unit();
                        return 0.5 * Colour::new(normal.x + 1.0, normal.y + 1.0, normal.z + 1.0, 1.0);
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
