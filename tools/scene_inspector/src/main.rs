//  MAIN.rs
//    by Lut99
//
//  Description:
//!   Entrypoint to the scene inspector.
//

use std::process::ExitCode;

use clap::Parser;
use error_trace::toplevel;
use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info};
use scene_inspector::App;
use winit::event_loop::EventLoop;


/***** CLI *****/
#[derive(Parser)]
struct Arguments {
    /// If given, enables DEBUG- and INFO-level log messages.
    #[clap(long, global = true)]
    debug: bool,
    /// If given, enables TRACE-level log messages. Implies `--debug`.
    #[clap(long, global = true)]
    trace: bool,
}





/***** ENTRYPOINT *****/
/// Main entrypoint for the scene inspector.
///
/// # Returns
/// The exit code of the application.
pub fn main() -> ExitCode {
    // Parse arguments
    let args = Arguments::parse();

    // Setup the logger
    if let Err(err) = HumanLogger::terminal(if args.trace {
        DebugMode::Full
    } else if args.debug {
        DebugMode::Debug
    } else {
        DebugMode::HumanFriendly
    })
    .init()
    {
        eprintln!("WARNING: Failed to setup logger: {err} (no logging for this session)");
    }
    info!("{} - v{}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));

    // Build the event loop
    debug!("Initializing event loop...");
    let event_loop = match EventLoop::with_user_event().build() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            error!("{}", toplevel!(("Failed to initialize event loop"), err));
            return ExitCode::FAILURE;
        },
    };

    // Initialize the app
    debug!("Initializing app...");
    let mut app = App::new();
    if let Err(err) = event_loop.run_app(&mut app) {
        error!("{}", toplevel!(("Failed to initialize app"), err));
        return ExitCode::FAILURE;
    }

    // Done
    info!("Done.");
    ExitCode::SUCCESS
}
