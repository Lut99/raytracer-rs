//  SPEC.rs
//    by Lut99
// 
//  Created:
//    29 Apr 2023, 10:55:28
//  Last edited:
//    29 Apr 2023, 11:11:53
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the public interfaces and structs that the `ecs` crate uses.
// 

use std::fmt::Debug;


/***** LIBRARY *****/
/// Defines the identifier for Entities.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Entity(pub(super) u64);



/// Defines a trait that abstracts over runtime identifiers for Components.
pub trait ComponentSalt {
    /// Returns some numeric representation of the salt. Which doesn't really matter, as long as it's distinct from the things you want to differentiate with.
    /// 
    /// # Returns
    /// An unsigned, 64-bit integer representing the variant.
    fn variant(&self) -> u64;
}
impl ComponentSalt for u64 {
    #[inline]
    fn variant(&self) -> u64 { *self }
}

/// Defines the trait that identifies Components.
pub trait Component: 'static + Debug {}
