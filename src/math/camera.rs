//  CAMERA.rs
//    by Lut99
// 
//  Created:
//    28 Apr 2023, 10:33:16
//  Last edited:
//    28 Apr 2023, 10:43:06
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the [`Camera`] class, which we can use to control the camera
//!   in a render scene.
// 

use super::vec3::Vec3;


/***** LIBRARY *****/
/// The Camera struct defines a camera and controls for managing it.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// Defines the focal point, i.e., where all rays start from.
    pub origin            : Vec3,
    /// Defines the horizontal axis of the viewport.
    pub horizontal        : Vec3,
    /// Defines the vertical axis of the viewport.
    pub vertical          : Vec3,
    /// Defines the lower left corner of the viewport.
    pub lower_left_corner : Vec3,
}

impl Camera {
    /// Constructor for the Camera that initializes at the origin (0, 0, 0), looking forward, with the given settings.
    /// 
    /// # Arguments
    /// - `viewport`: The logical `(width, height)` of the camera's viewport.
    /// - `focal_length`: The logical distance between the focal point (i.e., the eye) and the viewport. Essentially determines the "steepness" of the rays.
    /// 
    /// # Returns
    /// A new Camera instance derived from the given properties. 
    pub fn new(viewport: (f64, f64), focal_length: f64) -> Self {
        // Set some of the hardcoded settings
        let origin: Vec3 = Vec3::zeroes();

        // Compute the viewport's vectors
        let horizontal : Vec3 = Vec3::new(viewport.0, 0, 0);
        let vertical   : Vec3 = Vec3::new(0, viewport.1, 0);

        // Compute the lower left corner position of the vector (such that we can add the horizontal and vertical vectors)
        let lower_left_corner : Vec3 = origin - horizontal/2.0 - vertical/2.0 - Vec3::new(0, 0, focal_length);

        // Use that to create ourselves
        Self {
            origin,
            horizontal,
            vertical,
            lower_left_corner,
        }
    }
}
