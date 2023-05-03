//  PROGRESS.rs
//    by Lut99
// 
//  Created:
//    03 May 2023, 08:43:53
//  Last edited:
//    03 May 2023, 09:04:18
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements a fancy, multi-thread progress bar that can be used to
//!   show the progress of various threads, if need be.
// 

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::time::{Duration, Instant};

use atty::Stream;
use num_traits::NumAssign;


/***** ERRORS *****/
/// Defines errors originating from the ProgressBar.
#[derive(Debug)]
pub enum Error {

}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            
        }
    }
}





/***** LIBRARY *****/
/// The ProgressBar defines a singe progress bar that we can render to the terminal.
#[derive(Debug)]
pub struct ProgressBar<T> {
    /// Defines the current progress.
    size     : T,
    /// Defines the maximum amount to count to.
    capacity : T,

    /// Defines the last time we rendered.
    last_render     : Instant,
    /// Defines the render update interval, i.e., the timeout between renders.
    render_interval : Duration,
    /// Defines whether we render as a TUI or as a string of text.
    use_inplace     : bool,
    /// Defines whether we render with colour or not.
    use_colour      : bool,
}

impl<T: Copy + NumAssign + PartialOrd> ProgressBar<T> {
    /// Constructor for the Progressbar that initializes it to zero progress.
    /// 
    /// # Arguments
    /// - `capacity`: The maximum number of something (`T`) to count to.
    /// 
    /// # Returns
    /// A new ProgressBar that can be used to start keeping track of progress.
    #[inline]
    pub fn new(capacity: T) -> Self {
        let render_interval: Duration = Duration::from_millis(500);
        Self {
            size : T::zero(),
            capacity,

            last_render : Instant::now() - render_interval,
            render_interval,
            use_inplace : atty::is(Stream::Stdout),
            use_colour  : atty::is(Stream::Stdout),
        }
    }



    /// Factory-like method that sets the interval between two render calls.
    /// 
    /// Note that progressbar can be updated more frequently; this is just to avoid slowing down the app by rendering every time.
    /// 
    /// # Arguments
    /// - `interval`: The [`Duration`] that describes the time between render calls.
    /// 
    /// # Returns
    /// A mutable reference to this ProgressBar for optional chaining.
    #[inline]
    pub fn render_interval(&mut self, interval: Duration) -> &mut Self {
        self.render_interval = interval;
        self
    }

    /// Factory-like method that enforces a particular fanciness on the progressbar.
    /// 
    /// Specifically, if the fanciness is set, will print like a real progressbar that is updated in-place. Otherwise, will print as a series of percentage updates.
    /// 
    /// # Arguments
    /// - `use_inplace`: Whether to inplace rendering or not.
    /// 
    /// # Returns
    /// A mutable reference to this ProgressBar for optional chaining.
    #[inline]
    pub fn inplace(&mut self, use_inplace: bool) -> &mut Self {
        self.use_inplace = use_inplace;
        self
    }

    /// Factory-like method that enforces the use of colours on the progressbar.
    /// 
    /// # Arguments
    /// - `use_colour`: Whethre to use ANSI colours while rendering or not.
    /// 
    /// # Returns
    /// A mutable reference to this ProgressBar for optional chaining.
    #[inline]
    pub fn colour(&mut self, use_colour: bool) -> &mut Self {
        self.use_colour = use_colour;
        self
    }



    /// Updates the progress bar with a single step (increment).
    /// 
    /// # Errors
    /// This function may error if we failed to write to stdout (and a render was triggered).
    #[inline]
    pub fn step(&mut self) -> Result<(), Error> { self.update(T::one()) }

    /// Updates the progress bar with the given extra values.
    /// 
    /// # Arguments
    /// - `n`: The number of steps to increment in one go.
    /// 
    /// # Errors
    /// This function may error if we failed to write to stdout (and a render was triggered).
    #[inline]
    pub fn update(&mut self, n: T) -> Result<(), Error> {
        // Increment ourselves
        self.size += n;
        if self.size > self.capacity { self.size = self.capacity; }

        // Render the bar if we're past the render time _or_ we're complete
        if self.size >= self.capacity || self.last_render.elapsed() >= self.render_interval {
            self.render()
        } else {
            Ok(())
        }
    }

    /// Resets the progress bar to start.
    /// 
    /// Note that this function always forces a re-render.
    /// 
    /// # Errors
    /// This function may error if we failed to write to stdout.
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        self.size = T::zero();
        self.render()
    }



    /// Renders the progressbar to stdout.
    /// 
    /// # Errors
    /// This function may error if we failed to write to stdout.
    pub fn render(&self) -> Result<(), Error> {
        // Switch on how to render
        if self.use_inplace {

        } else {
            // Fallback to a much easier method
            
        }
    }
}
