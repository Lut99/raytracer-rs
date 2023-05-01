//  GROUP.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 18:56:39
//  Last edited:
//    01 May 2023, 19:19:25
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines a renderable group of objects, which we can use to consider
//!   multiple objects as one.
//!   
//!   Note that this distinction is actually purely virtual; at render time,
//!   object groups will be reduced to simple annotations in the list of
//!   objects.
// 

use serde::{Deserialize, Serialize};

use super::Object;
use super::spec::BoundingBoxable;
use super::utils::surround_list;


/***** LIBRARY *****/
/// An ObjectGroup defines a nested list of objects, grouped together for rendering or update purposes.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObjectGroup {
    /// The list of objects we are grouping.
    pub objects : Vec<Object>,
}

impl BoundingBoxable for ObjectGroup {
    #[inline]
    fn aabb(&self) -> crate::math::AABB { surround_list(&self.objects) }
}
