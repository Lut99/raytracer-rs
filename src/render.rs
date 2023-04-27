//  RENDER.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 14:40:55
//  Last edited:
//    27 Apr 2023, 14:46:06
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the main render functionality.
// 

use image::RgbaImage;
use log::info;

use crate::specifications::scene::SceneFile;


/***** LIBRARY *****/
/// Implements the main rendering functionality.
/// 
/// # Arguments
/// - `scene`: A [`SceneFile`] that describes what to render.
/// 
/// # Returns
/// A newly rendered image based on the given scene file.
pub fn handle(scene: SceneFile) -> RgbaImage {
    info!("Rendering scene...");

    

    // Done
    RgbaImage::new(0, 0)
}
