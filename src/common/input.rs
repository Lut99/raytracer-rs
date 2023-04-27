//  INPUT.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 12:59:41
//  Last edited:
//    27 Apr 2023, 13:07:01
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines structs and such that are used in parsing arguments or
//!   files.
// 

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::str::FromStr;


/***** ERRORS *****/
/// Defines errors that may occur when parsing a [`Dimensions`] struct.
#[derive(Debug)]
pub enum DimensionsParseError {
    /// Failed to find the separating `x`.
    MissingX { raw: String },
    /// Failed to parse the width as an unsigned integer.
    WidthParseFail{ raw: String, err: std::num::ParseIntError },
    /// Failed to parse the height as an unsigned integer.
    HeightParseFail{ raw: String, err: std::num::ParseIntError },
}
impl Display for DimensionsParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use DimensionsParseError::*;
        match self {
            MissingX{ raw }             => write!(f, "Cannot find `x` in dimensions '{raw}'"),
            WidthParseFail{ raw, err }  => write!(f, "Cannot parse width '{raw}' as an unsigned integer: {err}"),
            HeightParseFail{ raw, err } => write!(f, "Cannot parse height '{raw}' as an unsigned integer: {err}"),
        }
    }
}
impl Error for DimensionsParseError {}





/***** LIBRARY *****/
/// Defines an `<WIDTH>x<HEIGHT>` pair.
#[derive(Clone, Copy, Debug)]
pub struct Dimensions(pub u32, pub u32);

impl Display for Dimensions {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}x{}", self.0, self.1)
    }
}
impl FromStr for Dimensions {
    type Err = DimensionsParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Attempt to find the splitting 'x'
        let x_pos: usize = match s.find('x') {
            Some(pos) => pos,
            None      => { return Err(DimensionsParseError::MissingX { raw: s.into() }); },
        };

        // Get the parts
        let swidth  : &str = &s[..x_pos];
        let sheight : &str = &s[x_pos + 1..];

        // Parse them
        let width: u32 = match u32::from_str(swidth) {
            Ok(width) => width,
            Err(err)  => { return Err(DimensionsParseError::WidthParseFail { raw: swidth.into(), err }); },
        };
        let height: u32 = match u32::from_str(sheight) {
            Ok(height) => height,
            Err(err)   => { return Err(DimensionsParseError::HeightParseFail { raw: sheight.into(), err }); },
        };

        // Done, return the Dimensions
        Ok(Self(width, height))
    }
}

impl From<Dimensions> for (u32, u32) {
    #[inline]
    fn from(value: Dimensions) -> Self { (value.0, value.1) }
}
