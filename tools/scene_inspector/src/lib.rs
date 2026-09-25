//  LIB.rs
//    by Lut99
//
//  Description:
//!   Defines everything except the entrypoint of the scene_inspector.
//

// Modules
pub mod state;

// Imports
use std::sync::Arc;

use error_trace::toplevel;
use log::{debug, error, info};
pub use state::State;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};


/***** LIBRARY *****/
/// Defines the toplevel application struct that handles everything.
pub struct App {
    state: Option<State>,
}

// Constructors
impl App {
    /// Constructor for the App.
    ///
    /// # Returns
    /// A new instance of an App.
    pub const fn new() -> Self { Self { state: None } }
}

// Interfaces
impl ApplicationHandler<State> for App {
    #[inline]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("Resuming App...");

        // Create a new window for us
        debug!("Creating window...");
        let mut window_attrs = Window::default_attributes();
        window_attrs.active = true;
        window_attrs.visible = true;
        let window = match event_loop.create_window(window_attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                error!("{}", toplevel!(("Failed to create window"), err));
                return;
            },
        };

        // Create a new state with that window and that's it
        debug!("Creating state...");
        self.state = match pollster::block_on(State::new(window)) {
            Ok(state) => Some(state),
            Err(err) => {
                error!("{}", toplevel!(("Failed to create state"), err));
                return;
            },
        };
    }

    #[inline]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: State) {
        // Receive the new, updated state
        self.state = Some(event);
    }

    #[inline]
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        // Extract the internal state, or do nothing
        let Some(state) = &mut self.state else { return };

        // Match the event sent
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if let Err(err) = state.render() {
                    error!("{}", toplevel!(("Failed to run a render pass"), err));
                    event_loop.exit();
                }
            },
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(code), state: key_state, .. }, .. } => {
                state.handle_key(event_loop, code, key_state.is_pressed());
            },
            _ => return,
        }
    }
}
