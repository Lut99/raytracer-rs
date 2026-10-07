//  CAMERA CONTROLLER.rs
//    by Lut99
//
//  Description:
//!   Implements the part of the app that handles user input s.t. the camera
//!   moves.
//

use std::time::Duration;

use winit::keyboard::KeyCode;

use crate::math::Camera;


/***** LIBRARY *****/
pub struct CameraController {
    /// The speed with which we move.
    speed: f32,
    /// Which keys are pressed since last we looked.
    fwd:   bool,
    bwk:   bool,
    lft:   bool,
    rgt:   bool,
}

// Constructors
impl Default for CameraController {
    #[inline]
    fn default() -> Self { Self::new(5.0) }
}
impl CameraController {
    /// Constructor for the CameraController.
    ///
    /// # Arguments
    /// - `speed`: The speed with which the camera moves. Measured at distance units per second.
    ///
    /// # Returns
    /// A new CameraController.
    #[inline]
    pub const fn new(speed: f32) -> Self { Self { speed, fwd: false, bwk: false, lft: false, rgt: false } }
}

// Updates
impl CameraController {
    /// Handles a key (un)press.
    ///
    /// # Arguments
    /// - `code`: The [`KeyCode`] that was (un)pressed.
    /// - `is_pressed`: Whether it was actually pressed.
    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => self.fwd = is_pressed,
            KeyCode::KeyS | KeyCode::ArrowDown => self.bwk = is_pressed,
            KeyCode::KeyA | KeyCode::ArrowLeft => self.lft = is_pressed,
            KeyCode::KeyD | KeyCode::ArrowRight => self.rgt = is_pressed,
            _ => return,
        }
    }
}

// Effect
impl CameraController {
    /// Updates a given [`Camera`] to move based on the pressed keys.
    ///
    /// # Arguments
    /// - `since_last_update`: How much time has progressed since the last update.
    /// - `cam`: The [`Camera`] to update.
    pub fn update_camera(&self, since_last_update: Duration, cam: &mut Camera) {
        let speed = self.speed as f64 * since_last_update.as_secs_f64();
        let forward = cam.lookat - cam.lookfrom;
        let uforward = forward.unit();
        let forward_length = forward.length();

        if self.fwd && forward_length > speed {
            cam.lookfrom += speed * uforward;
        }
        if self.bwk {
            cam.lookfrom -= speed * uforward;
        }

        let right = uforward.cross(cam.up);
        // Don't forget to update the forward/backward in case of movement
        let forward = cam.lookat - cam.lookfrom;
        let forward_length = forward.length();

        if self.rgt {
            cam.lookfrom = cam.lookat - (forward + right * speed).unit() * forward_length;
        }
        if self.lft {
            cam.lookfrom = cam.lookat - (forward - right * speed).unit() * forward_length;
        }
    }
}
