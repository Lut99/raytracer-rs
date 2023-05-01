//  FEATURES.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 19:45:19
//  Last edited:
//    01 May 2023, 19:48:17
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the [`FeatureFile`], which is a file that configures which
//!   render features to enable or not.
// 

use serde::{Deserialize, Serialize};

use crate::common::file::impl_file;


/***** LIBRARY *****/
/// The FeatureFile determines which features to enable or not.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FeaturesFile {
    /// Whether to enable anti-aliasing or not.
    pub anti_aliasing : bool,
}

impl Default for FeaturesFile {
    #[inline]
    fn default() -> Self {
        Self {
            anti_aliasing : true,
        }
    }
}

impl_file!(FeaturesFile, serde_yaml);
