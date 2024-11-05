#![no_std]
#![deny(missing_docs)]

//! Zero-allocation grapheme cluster iterator.
//!
//! This crate provides a pure functional iterator for processing text into grapheme clusters
//! while maintaining zero-allocation guarantees. It properly handles:
//!
//! - Unicode grapheme clusters (including emoji and combining marks)
//! - ANSI escape sequences (with optional counting)
//! - Zero-width joiners (ZWJ)
//! - Regional indicators (flags)
//! - Right-to-Left (RTL) text with combining marks
//!
//! # Features
//!
//! - Zero heap allocations
//! - Efficient boundary detection using bit patterns
//! - Support for complex emoji sequences
//! - Optional ANSI sequence handling
//! - Compliant with Unicode Standard Annex #29
//!
//! # Example
//!
//! ```
//! use graphmemes::{GraphemeIterator, Result};
//!
//! # fn main() -> Result<()> {
//! // Process text with emoji and ANSI
//! let text = "\x1b[31mhello 👋\x1b[0m";
//! let graphemes: Vec<_> = GraphemeIterator::new(text, false)
//!     .collect::<Result<Vec<_>>>()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Implementation Details
//!
//! The crate uses a fixed-size buffer ([`MAX_GRAPHEME_SIZE`]) to handle grapheme clusters,
//! which is sufficient for even complex emoji sequences. Boundary detection is performed
//! using efficient bit patterns and follows the rules specified in Unicode Standard
//! Annex #29.
//!
//! ANSI sequences can optionally be counted as separate graphemes, which is useful
//! for terminal applications that need to process colored text.
//!
//! # No-std Support
//!
//! This crate is `no_std` compatible and makes no heap allocations. All operations
//! use fixed-size buffers and stack-only data structures.

mod error;
mod grapheme;
mod iter;

pub use error::{GraphemeError, Result};
pub use grapheme::{boundary, Grapheme};
pub use iter::GraphemeIterator;

/// Maximum number of code points in a grapheme cluster.
///
/// This constant defines the size of the fixed buffer used to store grapheme clusters.
/// The value 8 is chosen to accommodate complex emoji sequences while maintaining
/// reasonable stack usage.
///
/// Common sequences that fit within this limit:
/// - Basic emoji: 1-2 code points
/// - Emoji with skin tone: 2-3 code points
/// - Family emoji: 7-8 code points
/// - Flag emoji: 2 code points
/// - Characters with combining marks: 2-3 code points
pub(crate) const MAX_GRAPHEME_SIZE: usize = 8;
