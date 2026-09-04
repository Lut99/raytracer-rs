//  GROUP.rs
//    by Lut99
//
//  Description:
//!   Defines a logical group of objects.
//!
//!   This destinction exists purely for organisational purposes - e.g., to
//!   correctly translate a set of objects.
//!
//!   It is processed away by the time of AABB searching.
//

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::transforms::Transform;
use super::JsonObject;


/***** LIBRARY *****/
/// A logical grouping of objects.
///
/// Note that it does not influence AABB searching.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group {
    /// The list of objects in this group.
    #[serde(alias = "objects")]
    pub objs: Vec<JsonObject>,
    /// Defines any transformations on the object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,
}

// Interfaces
impl Loadable for Group {
    type Error = <JsonObject as Loadable>::Error;

    #[inline]
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error> {
        for obj in &mut self.objs {
            obj.load(dir)?;
        }
        Ok(())
    }
}
