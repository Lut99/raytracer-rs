//  RENDER.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 14:40:55
//  Last edited:
//    28 Apr 2023, 11:22:02
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the main render functionality.
// 

use image::RgbaImage;
use log::info;

use crate::specifications::scene::SceneFile;
use crate::math::colour::Colour;
use crate::math::vec3::{dot3, Vec3, Vector as _};
use crate::math::ray::Ray;
use crate::math::camera::Camera;


/***** HELPER FUNCTIONS *****/
/// Computes whether a ray hits a sphere or not.
/// 
/// # Arguments
/// - `center`: The center point of the sphere.
/// - `radius`: The radius of the sphere.
/// - `ray`: The [`Ray`] to check if it hits.
/// 
/// # Returns
/// true if the [`Ray`] hits, or false otherwise.
fn hit_sphere(center: Vec3, radius: f64, ray: Ray) -> bool {
    // Compute the distance between the origin of the ray and the center of the sphere
    let oc: Vec3 = ray.origin - center;

    // We compute `a`, `b` and `c` in the classic ABC-formula. This we do to find the intersections between the Ray (origin + t*direction) and the sphere (x^2 + y^2 + z^2 = r^2).
    // For more explanation, see the tutorial (<https://raytracing.github.io/books/RayTracingInOneWeekend.html#addingasphere/ray-sphereintersection>)
    let a: f64 = dot3(ray.direct, ray.direct);
    let b: f64 = 2.0 * dot3(oc, ray.direct);
    let c: f64 = dot3(oc, oc) - radius * radius;

    // Compute the discriminant only, since we're only interested in the number of roots
    // D < 0 -> no intersection, D == 0 -> one intersection (touching side), D > 0 -> two intersections (passing through)
    let d: f64 = b*b - 4.0*a*c;
    d > 0.0
}



/// Computes an Rgba quadruplet based on what the Ray hits.
/// 
/// # Arguments
/// - `ray`: The [`Ray`] who's colour to compute.
/// 
/// # Returns
/// A new [`Rgba`] struct that contains the matched colour.
fn ray_colour(ray: Ray) -> Colour {
    // If it hits the sphere, return the sphere colour
    if hit_sphere(Vec3::new(0, 0, -1), 0.5, ray) {
        return Colour::new(1, 0, 0, 1);
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
/// - `image`: The image to which we will render the scene.
/// - `scene`: A [`SceneFile`] that describes what to render.
/// 
/// # Returns
/// A newly rendered image based on the given scene file.
pub fn handle(image: &mut RgbaImage, _scene: SceneFile) {
    info!("Rendering scene...");

    // Let us define the camera (static, for now)
    let camera: Camera = Camera::new(((image.width() as f64 / image.height() as f64) * 2.0, 2.0), 1.0);

    // Let us fire all the rays (we go top-to-bottom)
    for y in (0..image.height()).rev() {
        for x in 0..image.width() {
            let image_dims: (u32, u32) = (image.width(), image.height());

            // Convert our pixel values to logical values
            let u: f64 = x as f64 / (image_dims.0 as f64 - 1.0);
            let v: f64 = y as f64 / (image_dims.1 as f64 - 1.0);

            // Define the Ray and cast it
            let ray    : Ray    = Ray::new(camera.origin, camera.lower_left_corner + u * camera.horizontal + v * camera.vertical - camera.origin);
            let colour : Colour = ray_colour(ray);

            // Write the colour to the image
            image[(x, image_dims.1 - 1 - y)] = colour.into();
        }
    }

    // Done
}
