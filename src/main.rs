//  MAIN.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:30:03
//  Last edited:
//    27 Apr 2023, 13:06:40
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
use log::{error, info};

use raytracer::common::errors::PrettyError as _;
use raytracer::common::input::Dimensions;
use raytracer::generate;


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
    #[clap(name="PATH", help="The path to the scene file which we want to render.")]
    path : PathBuf,
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
        RaytracerSubcommand::Render(_) => {
            todo!();
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