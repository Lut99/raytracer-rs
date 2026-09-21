//  ROTATE.rs
//    by Lut99
//
//  Description:
//!   Defines rotational translations.
//

use serde::{Deserialize, Serialize};

use super::super::objects::HitData;
use super::Transforming;
use crate::math::camera::degrees_to_radians;
use crate::math::{AABB, Ray, Vec3};


/***** HELPER FUNCTIONS *****/
/// Rotates a vector around the Y-axis.
#[inline]
fn rotate_x(vec: Vec3, sin_theta: f64, cos_theta: f64) -> Vec3 {
    Vec3::new(vec.x, cos_theta * vec.y - sin_theta * vec.z, sin_theta * vec.y + cos_theta * vec.z)
}
/// Rotates a vector back around the Y-axis.
#[inline]
fn rotate_x_back(vec: Vec3, sin_theta: f64, cos_theta: f64) -> Vec3 {
    Vec3::new(vec.x, cos_theta * vec.y + sin_theta * vec.z, -sin_theta * vec.y + cos_theta * vec.z)
}

/// Rotates a vector around the Y-axis.
#[inline]
fn rotate_y(vec: Vec3, sin_theta: f64, cos_theta: f64) -> Vec3 {
    Vec3::new(cos_theta * vec.x - sin_theta * vec.z, vec.y, sin_theta * vec.x + cos_theta * vec.z)
}
/// Rotates a vector back around the Y-axis.
#[inline]
fn rotate_y_back(vec: Vec3, sin_theta: f64, cos_theta: f64) -> Vec3 {
    Vec3::new(cos_theta * vec.x + sin_theta * vec.z, vec.y, -sin_theta * vec.x + cos_theta * vec.z)
}

/// Rotates a vector around the Z-axis.
#[inline]
fn rotate_z(vec: Vec3, sin_theta: f64, cos_theta: f64) -> Vec3 {
    Vec3::new(cos_theta * vec.x - sin_theta * vec.y, sin_theta * vec.x - cos_theta * vec.y, vec.z)
}
/// Rotates a vector back around the Y-axis.
#[inline]
fn rotate_z_back(vec: Vec3, sin_theta: f64, cos_theta: f64) -> Vec3 {
    Vec3::new(cos_theta * vec.x + sin_theta * vec.y, -sin_theta * vec.x + cos_theta * vec.y, vec.z)
}





/***** LIBRARY *****/
macro_rules! rotate_impl {
    () => {
        rotate_impl!(RotateX, rotate_x, rotate_x_back);
        rotate_impl!(RotateY, rotate_y, rotate_y_back);
        rotate_impl!(RotateZ, rotate_z, rotate_z_back);
    };

    ($name:ident, $rotate:ident, $rotate_back:ident) => {
        /// Implements rotation around one of the axis.
        #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
        pub struct $name {
            /// The angle, in degrees.
            pub angle: f64,
        }

        // Interfaces
        impl Transforming for $name {
            #[inline]
            fn transform_aabb(&self, _t_us: u64, aabb: AABB) -> AABB {
                // Compute the sin_theta and cos_theta for this angle
                let angle_radians: f64 = degrees_to_radians(self.angle);
                let sin_theta: f64 = angle_radians.sin();
                let cos_theta: f64 = angle_radians.cos();

                // Compute the translated points of the box and find min & max of those
                let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
                let mut max = Vec3::new(-f64::INFINITY, -f64::INFINITY, -f64::INFINITY);
                for i in 0..2 {
                    for j in 0..2 {
                        for k in 0..2 {
                            // Compute the original position of this corner
                            let pos = Vec3::new(
                                (i as f64 * aabb.x.max()) + (1.0 - i as f64) * aabb.x.min(),
                                (j as f64 * aabb.y.max()) + (1.0 - j as f64) * aabb.y.min(),
                                (k as f64 * aabb.z.max()) + (1.0 - k as f64) * aabb.z.min(),
                            );

                            // Rotate the point
                            let rotpos = $rotate_back(pos, sin_theta, cos_theta);

                            // Consider if it's a boundary
                            for c in 0..3 {
                                min[c] = f64::min(min[c], rotpos[c]);
                                max[c] = f64::max(max[c], rotpos[c]);
                            }
                        }
                    }
                }

                // Done, return the surrounding AABB
                AABB::from_points(min, max)
            }

            #[inline]
            fn transform(&self, _t_us: u64, ray: Ray) -> Ray {
                // Compute the sin_theta and cos_theta for this angle
                let angle_radians: f64 = degrees_to_radians(self.angle);
                let sin_theta: f64 = angle_radians.sin();
                let cos_theta: f64 = angle_radians.cos();

                // Transform the ray from world space to object space
                let origin = $rotate(ray.origin, sin_theta, cos_theta);
                let direct = $rotate(ray.direct, sin_theta, cos_theta);
                Ray::with_time(origin, direct, ray.time)
            }

            #[inline]
            fn transform_back(&self, _t_us: u64, mut rec: HitData) -> HitData {
                // Compute the sin_theta and cos_theta for this angle
                let angle_radians: f64 = degrees_to_radians(self.angle);
                let sin_theta: f64 = angle_radians.sin();
                let cos_theta: f64 = angle_radians.cos();

                // Transform the ray from object space to world space
                rec.hit = $rotate_back(rec.hit, sin_theta, cos_theta);
                rec.normal = $rotate_back(rec.normal, sin_theta, cos_theta);
                rec
            }
        }
    };
}
rotate_impl!();
