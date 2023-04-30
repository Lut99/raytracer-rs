//  SCENE.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:40:52
//  Last edited:
//    30 Apr 2023, 11:22:30
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the scene file.
// 

use serde::{Deserialize, Serialize};

use crate::common::file::impl_file;
use super::objects::Object;


/***** LIBRARY *****/
/// The SceneFile defines the scene's file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SceneFile {
    /// The objects found in this scene.
    pub objects : Vec<Object>,
}
impl_file!(SceneFile, serde_yaml);
