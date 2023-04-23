//  MAIN.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:30:03
//  Last edited:
//    23 Apr 2023, 11:38:48
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
use log::{debug, info};


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
}

/// Defines the arguments for the `render` subcommand.
#[derive(Debug, Parser)]
struct RenderArguments {
    /// The path to the scene file to render.
    #[clap(name="PATH", help="The path to the scene file which we want to render.")]
    path : PathBuf,
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

    
}
