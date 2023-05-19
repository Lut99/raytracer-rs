//  MAIN.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:30:03
//  Last edited:
//    19 May 2023, 12:53:51
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
use raytracer::render::spec::{RayRenderer as _, RenderBackend};
use raytracer::render::image::Image;
use raytracer::render::single::SingleThreadRenderer;
use raytracer::render::multi::{MultiThreadRenderer, MultiThreadRendererConfig};


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
    /// The output size of the image.
    #[clap(short, long, default_value="800x600", help="The size of the output image for this render.")]
    dims     : Dimensions,
    /// Whether to fix missing directories when generating the output image or not.
    #[clap(short, long, help="If given, will generate missing directories for the output image.")]
    fix_dirs : bool,

    /// The backend to use for rendering.
    #[clap(short, long, default_value="single", help="The backend to use for rendering.")]
    backend        : RenderBackend,
    /// Any additional config parameters to set for the backend file.
    #[clap(long, help="If given, defines a file that defines backend-specific properties.")]
    backend_config : Option<PathBuf>,

    /// The file defining a constant setting of features.
    #[clap(short='F', long, help="If given, will use the features enabled in the given features file.")]
    features_file : Option<PathBuf>,
    #[clap(flatten)]
    features : FeaturesCli,

    /// A once-more nested subcommand that defines what type of media to render.
    #[clap(subcommand)]
    media : RenderSubcommand,
}
/// Defines the subcommands for the `render` subcommand.
#[derive(Debug, EnumDebug, Subcommand)]
enum RenderSubcommand {
    /// Renders a single frame/image.
    #[clap(name = "image", alias = "frame", about = "Renders a single frame of the given scene.")]
    Image(RenderImageArguments),
}
/// Defines the arguments for the `render image` subcommand.
#[derive(Debug, Parser)]
struct RenderImageArguments {
    /// The path to the scene file to render.
    #[clap(name="SCENE_PATH", help="The path to the scene file which we want to render.")]
    scene_path  : PathBuf,
    /// The path to the image file to output.
    #[clap(name="OUTPUT_PATH", default_value="./image.png", help="The path to write the rendered image to.")]
    output_path : PathBuf,
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

            // Match further on the media type
            match render.media {
                RenderSubcommand::Image(image) => {
                    // Load the given scene file
                    debug!("Loading scene file '{}'...", image.scene_path.display());
                    let scene: SceneFile = match SceneFile::from_path(&image.scene_path) {
                        Ok(scene) => scene,
                        Err(err)  => { error!("{}", err.stack()); std::process::exit(1); },
                    };

                    // Convert that to a static HitList
                    let list: HitList = HitList::from(&scene.objects);

                    // Now render based on the backend
                    let output: Image = match render.backend {
                        RenderBackend::SingleThreaded => {
                            debug!("Rendering with single-threaded backend");
                            let renderer: SingleThreadRenderer = SingleThreadRenderer::new(render.dims.into(), features, true);
                            renderer.render_frame(&list).unwrap()
                        },

                        RenderBackend::MultiThreaded => {
                            debug!("Rendering with multi-threaded backend");

                            // Read the given file, if any
                            let config: MultiThreadRendererConfig = match render.backend_config {
                                Some(path) => {
                                    debug!("Loading multi-threaded backend file '{}'...", path.display());
                                    match MultiThreadRendererConfig::from_path(path) {
                                        Ok(config) => config,
                                        Err(err)   => { error!("{}", err.stack()); std::process::exit(1); },
                                    }
                                },
                                None => Default::default(),
                            };

                            // Create the backend
                            let renderer: MultiThreadRenderer = match MultiThreadRenderer::new(render.dims.into(), features, config) {
                                Ok(renderer) => renderer,
                                Err(err)     => { error!("{}", err.stack()); std::process::exit(1); },
                            };

                            // Now render with this backend
                            renderer.render_frame(&list).unwrap()
                        },
                    };

                    // Now write the image to disk
                    if let Err(err) = output.to_path(&image.output_path, render.fix_dirs) { error!("Failed to save rendered image to '{}': {}", image.output_path.display(), err); std::process::exit(1); }
                },
            }
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
