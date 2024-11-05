//! Error types for grapheme cluster processing.
//!
//! This module provides error types for handling invalid sequences and buffer overflows
//! that may occur during grapheme cluster iteration. All errors contain precise location
//! information and human-readable help messages.

use core::{error::Error, fmt};
use owo_colors::OwoColorize;

/// Errors that can occur during grapheme cluster iteration.
///
/// Each error variant includes the byte offset where the error occurred and
/// the length of the problematic sequence. This allows for precise error reporting
/// and debugging.
///
/// # Examples
///
/// ```
/// use graphmemes::GraphemeError;
///
/// let err = GraphemeError::invalid_ansi(10, 3);
/// assert_eq!(err.offset(), 10);
/// assert_eq!(err.sequence_length(), 3);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphemeError {
    /// Invalid ANSI escape sequence encountered.
    ///
    /// This error occurs when an ANSI escape sequence contains invalid characters
    /// or is malformed. ANSI sequences must contain only ASCII characters.
    InvalidAnsiSequence {
        /// Starting byte offset of the invalid sequence
        offset: usize,
        /// Length in bytes of the invalid sequence
        sequence_len: usize,
    },

    /// Grapheme cluster exceeds maximum buffer size.
    ///
    /// This error occurs when a grapheme cluster would require more than
    /// [`MAX_GRAPHEME_SIZE`](crate::MAX_GRAPHEME_SIZE) code points to represent.
    BufferOverflow {
        /// Starting byte offset where overflow occurred
        offset: usize,
        /// Length in bytes of the sequence that caused overflow
        sequence_len: usize,
    },
}

impl GraphemeError {
    /// Creates a new `InvalidAnsiSequence` error.
    ///
    /// # Arguments
    ///
    /// * `offset` - The byte offset where the invalid sequence starts
    /// * `sequence_len` - The length in bytes of the invalid sequence
    ///
    /// # Examples
    ///
    /// ```
    /// use graphmemes::GraphemeError;
    ///
    /// let err = GraphemeError::invalid_ansi(5, 2);
    /// ```
    #[inline]
    pub fn invalid_ansi(offset: usize, sequence_len: usize) -> Self {
        Self::InvalidAnsiSequence {
            offset,
            sequence_len,
        }
    }

    /// Creates a new `BufferOverflow` error.
    ///
    /// # Arguments
    ///
    /// * `offset` - The byte offset where the overflow occurred
    /// * `sequence_len` - The length in bytes of the sequence that caused the overflow
    ///
    /// # Examples
    ///
    /// ```
    /// use graphmemes::GraphemeError;
    ///
    /// let err = GraphemeError::buffer_overflow(10, 4);
    /// ```
    #[inline]
    pub fn buffer_overflow(offset: usize, sequence_len: usize) -> Self {
        Self::BufferOverflow {
            offset,
            sequence_len,
        }
    }

    /// Returns the byte offset where the error occurred.
    ///
    /// This offset represents the position in the input string where
    /// the problematic sequence begins.
    #[inline]
    pub fn offset(&self) -> usize {
        match self {
            Self::InvalidAnsiSequence { offset, .. } | Self::BufferOverflow { offset, .. } => {
                *offset
            }
        }
    }

    /// Returns the length in bytes of the problematic sequence.
    ///
    /// For invalid ANSI sequences, this is the length of the malformed sequence.
    /// For buffer overflows, this is the length of the sequence that would
    /// exceed the buffer size.
    #[inline]
    pub fn sequence_length(&self) -> usize {
        match self {
            Self::InvalidAnsiSequence { sequence_len, .. }
            | Self::BufferOverflow { sequence_len, .. } => *sequence_len,
        }
    }

    /// Returns a human-readable error message.
    ///
    /// This message describes the error condition without any formatting
    /// or color.
    #[inline]
    pub const fn message(&self) -> &'static str {
        match self {
            Self::InvalidAnsiSequence { .. } => "Invalid ANSI sequence",
            Self::BufferOverflow { .. } => "Grapheme buffer overflow",
        }
    }

    /// Returns a helpful description of how to fix the error.
    ///
    /// This message provides guidance on how to correct the condition
    /// that caused the error.
    #[inline]
    pub const fn help(&self) -> &'static str {
        match self {
            Self::InvalidAnsiSequence { .. } => "ANSI sequences must follow format: \\x1b[<n>m",
            Self::BufferOverflow { .. } => {
                "Grapheme sequence exceeds maximum supported length (8 code points)"
            }
        }
    }
}

impl fmt::Display for GraphemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = self.message();
        let help = self.help();

        let colored_msg = msg.red();
        let colored_msg_bold = colored_msg.bold();
        let colored_help = help.yellow();

        write!(f, "{}", colored_msg_bold)?;
        write!(
            f,
            " at offset {} (sequence length {})",
            self.offset(),
            self.sequence_length()
        )?;
        write!(f, "\nHelp: {}", colored_help)
    }
}

impl Error for GraphemeError {}

/// A specialized Result type for grapheme operations.
pub type Result<T> = core::result::Result<T, GraphemeError>;

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::string::String;
    use core::fmt::Write;

    // Helper function to create a test string without format!
    fn make_test_string(err: &GraphemeError) -> String {
        let mut s = String::new();
        let _ = write!(&mut s, "{}", err);
        s
    }

    #[test]
    fn test_error_creation() {
        let err = GraphemeError::invalid_ansi(10, 3);
        assert_eq!(err.offset(), 10);
        assert_eq!(err.sequence_length(), 3);

        let err = GraphemeError::buffer_overflow(20, 5);
        assert_eq!(err.offset(), 20);
        assert_eq!(err.sequence_length(), 5);
    }

    #[test]
    fn test_error_messages() {
        let err = GraphemeError::invalid_ansi(0, 1);
        assert!(err.message().contains("ANSI"));
        assert!(err.help().contains("\\x1b"));

        let err = GraphemeError::buffer_overflow(0, 1);
        assert!(err.message().contains("buffer"));
        assert!(err.help().contains("maximum"));
    }

    #[test]
    fn test_error_display() {
        let err = GraphemeError::invalid_ansi(5, 2);
        let display = make_test_string(&err);
        assert!(display.contains("offset 5"));
        assert!(display.contains("sequence length 2"));
        assert!(display.contains("Help:"));
    }

    #[test]
    fn test_error_debug() {
        let err = GraphemeError::buffer_overflow(10, 3);
        let debug = make_test_string(&err);
        assert!(debug.contains("10"));
        assert!(debug.contains("3"));
    }

    #[test]
    fn test_clone_and_eq() {
        let err1 = GraphemeError::invalid_ansi(1, 2);
        let err2 = err1;
        assert_eq!(err1, err2);

        let err3 = GraphemeError::invalid_ansi(1, 3);
        assert_ne!(err1, err3);
    }
}
