//  FEATURES.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 19:45:19
//  Last edited:
//    05 May 2023, 11:40:38
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the [`FeatureFile`], which is a file that configures which
//!   render features to enable or not.
// 

use serde::{Deserialize, Serialize};

use crate::common::file::impl_file;


/***** HELPER FUNCTIONS *****/
/// Returns `50` as a [`usize`].
#[inline]
pub const fn usize_50() -> usize { 50 }

/// Returns `100` as a [`usize`].
#[inline]
pub const fn usize_100() -> usize { 100 }





/***** LIBRARY *****/
/// The FeatureFile determines which features to enable or not.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FeaturesFile {
    /// Whether to enable anti-aliasing or not. Specifically, aliasing is enabled if the number of samples > 1.
    #[serde(alias="anti_aliasing", default="usize_100")]
    pub n_samples : usize,

    /// How many times we bounce a Ray, at most.
    #[serde(alias="bounce_depth", default="usize_50")]
    pub max_depth : usize,
}

impl Default for FeaturesFile {
    #[inline]
    fn default() -> Self {
        Self {
            n_samples : 100,

            max_depth : 50,
        }
    }
}

impl_file!(FeaturesFile, serde_yaml);
