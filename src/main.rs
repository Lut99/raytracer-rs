//  MAIN.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:30:03
//  Last edited:
//    06 May 2023, 11:14:23
//  Auto updated?
//    Yes
// 
//  Description:
//!   Entrypoint to the main `raytracer` application.
// 

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use enum_debug::EnumDebug;
use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info};

use raytracer::common::errors::PrettyError as _;
use raytracer::common::file::File as _;
use raytracer::common::input::Dimensions;
use raytracer::specifications::features::{Features, FeaturesCli, FeaturesFile};
use raytracer::specifications::scene::SceneFile;
use raytracer::hitlist::HitList;
use raytracer::generate;
use raytracer::render::frame;
use raytracer::render::image::Image;


/***** ARGUMENTS *****/
/// Defines the arguments for the `raytracer` application.
#[derive(Debug, Parser)]
struct Arguments {
    /// Whether to set [`DebugMode::Debug`] instead of [`DebugMode::HumanFriendly`].
    #[clap(long, global=true, help="If given, will enable additional debug prints (at the `info` and `debug` log level). Also makes the `warning` and `error` prints more extensive.")]
    debug : bool,
    /// Whether to set [`DebugMode::Full`] instead of [`DebugMode::HumanFriendly`].
    #[clap(long, global=true, help="If given, will enable most verbose debug prints. Implies `--debug`.")]
    trace : bool,

    /// The particular subcommand to select.
    #[clap(subcommand)]
    subcommand : RaytracerSubcommand,
}



/// Defines subcommands for the `raytracer` application.
#[derive(Debug, EnumDebug, Subcommand)]
enum RaytracerSubcommand {
    /// Renders a new scene.
    #[clap(name = "render", about = "Renders a particular scene.")]
    Render(RenderArguments),
    /// Generates something.
    #[clap(name = "generate", about = "Generates files for testing or for rendering.")]
    Generate(GenerateArguments),
}

/// Defines the arguments for the `render` subcommand.
#[derive(Debug, Parser)]
struct RenderArguments {
    /// The path to the scene file to render.
    #[clap(name="SCENE_PATH", help="The path to the scene file which we want to render.")]
    scene_path  : PathBuf,
    /// The path to the image file to output.
    #[clap(name="OUTPUT_PATH", default_value="./image.png", help="The path to write the rendered image to.")]
    output_path : PathBuf,

    /// The output size of the image.
    #[clap(short, long, default_value="800x600", help="The size of the output image for this render.")]
    dims     : Dimensions,
    /// Whether to fix missing directories when generating the output image or not.
    #[clap(short, long, help="If given, will generate missing directories for the output image.")]
    fix_dirs : bool,

    /// The file defining a constant setting of features.
    #[clap(short='F', long, help="If given, will use the features enabled in the given features file.")]
    features_file : Option<PathBuf>,
    #[clap(flatten)]
    features : FeaturesCli,
    // /// Whether to disable antialiasing or not.
    // #[clap(long, help="If given, will not implement anti-aliasing (i.e., does not send multiple rays per pixel). Shortcut for '--rays-per-pixel 1'")]
    // disable_anti_aliasing : bool,
    // /// The number of rays to shoot per pixel.
    // #[clap(long, help="Sets the number of rays to shoot per pixel. Setting '1' implies disabling antialiasing.")]
    // rays_per_pixel        : Option<usize>,
    // /// Sets the maximum bounce depth.
    // #[clap(long, help="The maximum times that a ray can bounce between objects.")]
    // max_depth             : Option<usize>,
}

/// Defines the arguments for the `generate` subcommand.
#[derive(Debug, Parser)]
struct GenerateArguments {
    /// Whether to create missing directories or not.
    #[clap(short, long, global=true, help="If given, generates missing directories instead of erroring.")]
    fix_dirs : bool,

    /// The thing to generate.
    #[clap(subcommand)]
    subcommand : GenerateSubcommand,
}
/// Defines the things we can generate.
#[derive(Debug, EnumDebug, Subcommand)]
enum GenerateSubcommand {
    #[clap(name = "gradient", about = "Generates the test gradient image discussed in the tutorial.")]
    Gradient {
        /// The output path where to generate the file to.
        #[clap(name="PATH", default_value="./image.png", help="The output path to generate the file to.")]
        path : PathBuf,
        /// The dimensions of the image, given as `WIDTHxHEIGHT`.
        #[clap(name="DIMENSIONS", default_value="256x256", help="The dimensions of the output image. Should be given as a `<WIDTH>x<HEIGHT>` pair, where `<WIDTH>` is the image's width, and `<HEIGHT>` is the image's height.")]
        dims : Dimensions,
    }
}





/***** ENTRYPOINT *****/
fn main() {
    // Read the command-line arguments
    let args: Arguments = Arguments::parse();

    // Setup the logger
    if let Err(err) = HumanLogger::terminal(DebugMode::from_flags(args.trace, args.debug)).init() {
        eprintln!("WARNING: Failed to setup logger: {err} (no logging enabled for this session)");
    }
    info!("raytracer-rs v{}", env!("CARGO_PKG_VERSION"));

    // Match on the subcommand
    match args.subcommand {
        RaytracerSubcommand::Render(render) => {
            // Load the given feature file, if any.
            let features: Option<FeaturesFile> = render.features_file.map(|p| {
                match FeaturesFile::from_path(&p) {
                    Ok(features) => features,
                    Err(err)     => { error!("{}", err.stack()); std::process::exit(1); },
                }
            });
            // Override it with other options
            let features: Features = Features::new(features, render.features);

            // Load the given scene file
            debug!("Loading scene file '{}'...", render.scene_path.display());
            let scene: SceneFile = match SceneFile::from_path(&render.scene_path) {
                Ok(scene) => scene,
                Err(err)  => { error!("{}", err.stack()); std::process::exit(1); },
            };

            // Convert that to a static HitList
            let list: HitList = HitList::from(&scene.objects);

            // Create the image with the target size and render to it
            let mut image: Image = Image::new(render.dims.into());
            frame::render(&mut image, &list, &features);

            // Now write the image to disk
            if let Err(err) = image.to_path(&render.output_path, render.fix_dirs) { error!("Failed to save rendered image to '{}': {}", render.output_path.display(), err); std::process::exit(1); }
        },

        RaytracerSubcommand::Generate(generate) => {
            // Further match
            match generate.subcommand {
                GenerateSubcommand::Gradient { path, dims } => {
                    // Run the command
                    if let Err(err) = generate::gradient(path, dims.into(), generate.fix_dirs) { error!("{}", err.stack()); std::process::exit(1); }
                },
            }
        }
    }
}
