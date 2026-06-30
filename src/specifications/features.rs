//  FEATURES.rs
//    by Lut99
//
//  Created:
//    01 May 2023, 19:45:19
//  Last edited:
//    06 May 2023, 12:02:20
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the [`FeatureFile`], which is a file that configures which
//!   render features to enable or not.
//

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::common::file::{impl_toml_from_path, impl_toml_from_string, impl_toml_to_path, impl_toml_to_string};


/***** LIBRARY *****/
/// The FeatureFile determines which features to enable or not.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FeaturesFile {
    /// Whether to correct for gamma or not.
    #[serde(alias = "gamma")]
    pub gamma_correction: Option<bool>,
    /// How many times we bounce a Ray, at most.
    #[serde(alias = "bounce_depth")]
    pub max_depth: Option<usize>,
}
impl FeaturesFile {
    impl_toml_from_string!();
    impl_toml_to_string!();
    impl_toml_from_path!();
    impl_toml_to_path!();
}



/// The FeaturesCli struct defines the CLI interface.
#[derive(Clone, Copy, Debug, Parser)]
pub struct FeaturesCli {
    /// Whether to enable gamma correction (or rather, to disable it).
    #[clap(long, help = "If given, disables gamma correction")]
    disable_gamma_correction: bool,

    /// Whether to enable anti-aliasing (or rather, to disable it).
    #[clap(long, help = "If given, disables anti-aliasing (shorthand for '--anti-aliasing-rays 1')")]
    disable_anti_aliasing: bool,
    /// Determines the number of rays to cast per pixel.
    #[clap(
        long,
        help = "The number of rays to cast per pixel. Setting to '1' implies disabling anti-aliasing. If omitted, uses the value from the features \
                file (or the default '100')."
    )]
    anti_aliasing_rays: Option<usize>,
    /// Determines the number of times a ray may bounce at most.
    #[clap(
        long,
        help = "The number of times a ray may bounce at most. Setting to '1' implies not bouncing anything ever (i.e., direct illumination), and \
                setting to '0' not even fires the ray. If omitted, uses the value from the features file (or the default '50')."
    )]
    ray_max_depth: Option<usize>,
}



/// The `Features` struct is an abstraction over a features file that combines it and any overrides from the CLI.
#[derive(Clone, Copy, Debug)]
pub struct Features {
    /// Whether to correct for gamma or not.
    pub gamma_correction: bool,
    /// The number of times we bounce a ray at maximum.
    pub max_depth: usize,
}

impl Default for Features {
    #[inline]
    fn default() -> Self { Self { gamma_correction: true, max_depth: 50 } }
}
impl Features {
    /// Constructor for the Features that constructs it from an optional features file and the CLI values.
    ///
    /// # Arguments
    /// - `file`: The [`FeaturesFile`] to use as fallback if the CLI does not mention it (if any).
    /// - `cli`: The [`FeaturesCli`] that determines what of the [`FeaturesFile`] to override.
    ///
    /// # Returns
    /// A new Features that holds a usable combination of both inputs.
    #[inline]
    pub fn new(file: Option<FeaturesFile>, cli: FeaturesCli) -> Self {
        // Get a default variant of the features if it fails
        let def: Self = Self::default();

        // Comput the join of that one and the features file
        let file: Self = match file {
            Some(file) => {
                Self { gamma_correction: file.gamma_correction.unwrap_or(def.gamma_correction), max_depth: file.max_depth.unwrap_or(def.max_depth) }
            },
            None => def,
        };

        // Finally, add in the CLI
        Self {
            gamma_correction: if cli.disable_gamma_correction { false } else { file.gamma_correction },
            max_depth: cli.ray_max_depth.unwrap_or(file.max_depth),
        }
    }
}
