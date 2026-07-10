//  VERTICAL.rs
//    by Lut99
//
//  Description:
//!   A simple animation that moves objects upwards along a line.
//

use serde::{Deserialize, Serialize};

use super::Animating;
use crate::math::Vec3;


/***** LIBRARY *****/
/// A simple animation that moves objects upwards along a line.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Vertical {
    /// The amount that we travel in logical distance...
    pub len: f64,
    /// ...at the given time...
    pub at: u64,
    /// ...over the given period of time.
    pub duration: u64,
}

impl Animating for Vertical {
    #[inline]
    fn animate(&self, pos: Vec3, t: u64) -> Vec3 {
        if t <= self.at {
            pos
        } else if t < self.at + self.duration {
            Vec3::new(pos.x, pos.y + ((t - self.at) as f64 / self.duration as f64) * self.len, pos.z)
        } else {
            Vec3::new(pos.x, pos.y + self.len, pos.z)
        }
    }
}
