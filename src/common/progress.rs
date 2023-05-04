//  PROGRESS.rs
//    by Lut99
// 
//  Created:
//    03 May 2023, 08:43:53
//  Last edited:
//    04 May 2023, 19:19:50
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements a fancy, multi-thread progress bar that can be used to
//!   show the progress of various threads, if need be.
// 

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::io::{stderr, stdout, Stderr, Stdout, Write};
use std::time::{Duration, Instant};

use atty::Stream;
use num_traits::NumAssign;


/***** ERRORS *****/
/// Defines errors originating from the ProgressBar.
#[derive(Debug)]
pub enum Error {
    /// Failed to write to whatever stdout we're attached to.
    Writer{ what: &'static str, err: std::io::Error },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            Writer{ what, .. } => write!(f, "Failed to render progressbar to {what}"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            Writer{ err, .. } => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// The ProgressBar defines a singe progress bar that we can render to the terminal.
#[derive(Debug)]
pub struct ProgressBar<W, T> {
    /// Defines the writer to which this ProgressBar writes.
    writer : W,

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
    /// The width in which we render the progress bar if in interactive mode. [`None`] means to choose the terminal width.
    width           : Option<usize>,
}

impl<T: Copy + NumAssign + PartialOrd> ProgressBar<Stdout, T> {
    /// Constructor for the ProgressBar that initializes it to zero progress and writes it to `stdout`.
    /// 
    /// # Arguments
    /// - `capacity`: The maximum number of something (`T`) to count to.
    /// 
    /// # Returns
    /// A new ProgressBar that can be used to start keeping track of progress.
    #[inline]
    pub fn stdout(capacity: T) -> Self {
        let is_atty: bool = atty::is(Stream::Stdout);
        Self::new(
            stdout(),
            capacity,
            Duration::from_millis(500),
            is_atty,
            is_atty,
            None,
        )
    }
}
impl<T: Copy + NumAssign + PartialOrd> ProgressBar<Stderr, T> {
    /// Constructor for the ProgressBar that initializes it to zero progress and writes it to `stderr`.
    /// 
    /// # Arguments
    /// - `capacity`: The maximum number of something (`T`) to count to.
    /// 
    /// # Returns
    /// A new ProgressBar that can be used to start keeping track of progress.
    #[inline]
    pub fn stderr(capacity: T) -> Self {
        let is_atty: bool = atty::is(Stream::Stderr);
        Self::new(
            stderr(),
            capacity,
            Duration::from_millis(500),
            is_atty,
            is_atty,
            None,
        )
    }
}
impl<W: Write, T: Copy + NumAssign + PartialOrd> ProgressBar<W, T> {
    /// Constructor for the Progressbar that initializes it to zero progress.
    /// 
    /// # Arguments
    /// - `writer`: The [`Write`]-enabled type to write to the progressbar to on [`ProgressBar::render()`] calls.
    /// - `capacity`: The maximum number of something (`T`) to count to.
    /// - `render_interval`: The duration between two render ticks. Note that the progressbar can be updated more than this; this interval is here to avoid the relatively expensive render on every update.
    /// - `use_inplace`: Whether to render "in-place" (i.e., update parts of the terminal to animate the progressbar) or to instead use a non-replacing alternative.
    /// - `use_colour`: Whether to use ANSI colours for the progressbar.
    /// - `width`: The width of the rendered progressbar. If [`None`], then the terminal width is used. Ignored if `use_inplace` is false.
    /// 
    /// # Returns
    /// A new ProgressBar that can be used to start keeping track of progress.
    #[inline]
    pub fn new(writer: W, capacity: T, render_interval: Duration, use_inplace: bool, use_colour: bool, width: Option<usize>) -> Self {
        Self {
            writer,

            size : T::zero(),
            capacity,

            last_render : Instant::now() - render_interval,
            render_interval,

            use_inplace,
            use_colour,
            width,
        }
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
            // Harder but much pretty method
            Ok(())
        } else {
            // Fallback to a much easier method
            Ok(())
        }
    }
}
