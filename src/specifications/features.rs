//  FEATURES.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 19:45:19
//  Last edited:
//    03 May 2023, 08:29:17
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the [`FeatureFile`], which is a file that configures which
//!   render features to enable or not.
// 

use num_traits::One as _;
use serde::{Deserialize, Serialize};

use crate::common::file::impl_file;


/***** LIBRARY *****/
/// The FeatureFile determines which features to enable or not.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FeaturesFile {
    /// Whether to enable anti-aliasing or not. Specifically, aliasing is enabled if the number of samples > 1.
    #[serde(alias="anti_aliasing", default="usize::one")]
    pub n_samples : usize,
}

impl Default for FeaturesFile {
    #[inline]
    fn default() -> Self {
        Self {
            n_samples : 100,
        }
    }
}

impl_file!(FeaturesFile, serde_yaml);
