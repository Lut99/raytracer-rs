//  SCENE.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:40:52
//  Last edited:
//    27 Apr 2023, 12:15:07
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the scene file.
// 

use serde::{Deserialize, Serialize};

use crate::common::file::impl_file;


/***** LIBRARY *****/
/// The SceneFile defines the scene's file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SceneFile {
    
}
impl_file!(SceneFile, serde_yaml);
