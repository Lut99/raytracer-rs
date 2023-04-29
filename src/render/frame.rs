//  FRAME.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 14:40:55
//  Last edited:
//    29 Apr 2023, 10:49:12
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the functionality to render a single frame based on a
//!   [`SceneFile`].
// 

use log::info;

use crate::specifications::scene::{Object, SceneFile, Sphere};
use crate::math::colour::Colour;
use crate::math::vec3::{dot3, Vec3, Vector as _};
use crate::math::ray::Ray;
use crate::math::camera::Camera;
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
/// - `objects`: The list of objects found in the scene file that we want to render.
/// 
/// # Returns
/// A new [`Rgba`] struct that contains the matched colour.
fn ray_colour(ray: Ray, objects: &[Object]) -> Colour {
    // If it hits the sphere, return the sphere colour
    for obj in objects {
        match obj {
            Object::Sphere(sphere) => if let Some(point) = hit_sphere(ray, sphere) {
                // Compute the normal
                let normal: Vec3 = (ray.at(point) - sphere.center).unit();
                return 0.5 * Colour::new(normal.x + 1.0, normal.y + 1.0, normal.z + 1.0, 1.0);
            }
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
/// - `scene`: A [`SceneFile`] that describes what to render.
/// 
/// # Returns
/// A newly rendered image based on the given scene file.
pub fn render(image: &mut Image, scene: SceneFile) {
    info!("Rendering scene...");

    // Let us define the camera (static, for now)
    let camera: Camera = Camera::new(((image.width() as f64 / image.height() as f64) * 2.0, 2.0), 1.0);

    // Let us fire all the rays (we go top-to-bottom)
    for ((x, y), ray) in RayGenerator::new(image.dims(), camera).coords() {
        // Compute the colour of the Ray
        let colour : Colour = ray_colour(ray, &scene.objects);

        // Write the colour to the image
        image[(x, y)] = colour.into();
    }

    // Done
}
