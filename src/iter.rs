//! Iterator implementation for Unicode grapheme clusters.
//!
//! This module provides a zero-allocation iterator that processes text into grapheme clusters
//! while properly handling:
//! - Unicode combining marks
//! - Emoji sequences including ZWJ (Zero Width Joiner) combinations
//! - ANSI escape sequences
//! - Regional indicators (flags)
//! - RTL (Right-to-Left) text
//!
//! The implementation follows Unicode Standard Annex #29 (UAX #29) for grapheme cluster
//! boundaries and supports extended grapheme clusters.

use crate::{boundary, grapheme::is_emoji, Grapheme, GraphemeError, Result, MAX_GRAPHEME_SIZE};
use core::str::Chars;

/// Bit mask for extracting the state bits from the state byte
const STATE_MASK: u8 = 0b111;
/// Initial state for the iterator
const STATE_START: u8 = 0;
/// State indicating the iterator is processing a grapheme cluster
const STATE_IN_GRAPHEME: u8 = 1;
/// State indicating the iterator is processing an ANSI escape sequence
const STATE_IN_ANSI: u8 = 2;

/// Zero-allocation iterator for Unicode grapheme clusters.
///
/// This iterator processes text into grapheme clusters following Unicode Standard Annex #29
/// rules for extended grapheme cluster boundaries. It handles:
///
/// - Basic ASCII text
/// - Complex Unicode sequences including combining marks
/// - Emoji sequences including skin tone modifiers
/// - Zero Width Joiner (ZWJ) sequences
/// - ANSI escape sequences (optionally counted as graphemes)
///
/// # Examples
///
/// ```
/// use graphmemes::{GraphemeIterator, Result};
///
/// # fn main() -> Result<()> {
/// let text = "Hello 👋🏽";
/// let graphemes: Vec<_> = GraphemeIterator::new(text, false).collect::<Result<_>>()?;
/// assert_eq!(graphemes.len(), 7); // "H" "e" "l" "l" "o" " " "👋🏽"
/// # Ok(())
/// # }
/// ```
///
/// With ANSI sequences:
///
/// ```
/// use graphmemes::{GraphemeIterator, Result};
///
/// # fn main() -> Result<()> {
/// let text = "\x1b[31mred\x1b[0m";
/// let graphemes: Vec<_> = GraphemeIterator::new(text, true).collect::<Result<_>>()?;
/// assert_eq!(graphemes.len(), 5); // "\x1b[31m" "r" "e" "d" "\x1b[0m"
/// # Ok(())
/// # }
/// ```
pub struct GraphemeIterator<'a> {
    /// Character iterator over the input text
    chars: Chars<'a>,
    /// Current byte position in the input string
    position: usize,
    /// Fixed-size buffer for accumulating grapheme clusters
    buffer: [char; MAX_GRAPHEME_SIZE],
    /// Number of characters currently in the buffer
    buffer_len: usize,
    /// Combined state byte: lower 3 bits for state, bit 3 for ANSI counting
    state: u8,
    /// Previous character category for boundary detection
    prev_category: u32,
}

impl<'a> GraphemeIterator<'a> {
    /// Creates a new grapheme cluster iterator.
    ///
    /// # Arguments
    ///
    /// * `text` - The input text to iterate over
    /// * `count_ansi` - Whether to count ANSI escape sequences as graphemes
    ///
    /// # Examples
    ///
    /// ```
    /// use graphmemes::GraphemeIterator;
    ///
    /// let text = "Hello, world!";
    /// let iter = GraphemeIterator::new(text, false);
    /// ```
    #[inline]
    pub fn new(text: &'a str, count_ansi: bool) -> Self {
        Self {
            chars: text.chars(),
            position: 0,
            buffer: ['\0'; MAX_GRAPHEME_SIZE],
            buffer_len: 0,
            state: STATE_START | ((count_ansi as u8) << 3),
            prev_category: 0,
        }
    }

    /// Returns whether ANSI sequences are being counted as graphemes.
    #[inline]
    fn count_ansi(&self) -> bool {
        (self.state >> 3) & 1 == 1
    }

    /// Returns the current processing state.
    #[inline]
    fn state(&self) -> u8 {
        self.state & STATE_MASK
    }

    /// Sets a new processing state while preserving ANSI counting flag.
    #[inline]
    fn set_state(&mut self, new_state: u8) {
        self.state = (self.state & !STATE_MASK) | new_state;
    }

    /// Determines if a character boundary exists before the given character.
    ///
    /// Implements UAX #29 grapheme cluster boundary rules.
    #[inline]
    fn is_boundary(&mut self, c: char) -> bool {
        let category = Grapheme::char_category(c);

        // Special case: first character is never a boundary
        if self.buffer_len <= 1 {
            self.prev_category = category;
            return false;
        }

        let is_boundary = match (self.prev_category, category) {
            // ZWJ sequences
            (_, boundary::ZWJ) => false,
            (boundary::ZWJ, _) if is_emoji(c) => false,

            // Extend characters never form boundary
            (_, boundary::EXTEND) => false,

            // Regional indicators must pair
            (boundary::REGIONAL, boundary::REGIONAL) => false,

            // Emoji modifiers don't form boundary
            (_, boundary::EMOJI_MOD) => false,

            // SpacingMarks don't form boundary
            (_, boundary::SPACINGMARK) => false,

            // Prepend doesn't form boundary
            (boundary::PREPEND, _) => false,

            // Everything else is a boundary
            _ => true,
        };

        // Update category state
        self.prev_category = category;
        is_boundary
    }

    /// Processes a single character, potentially producing a complete grapheme.
    ///
    /// This function implements a state machine that handles:
    /// - Fast path for standalone ASCII characters
    /// - ANSI escape sequences
    /// - Complex grapheme clusters
    ///
    /// The state transitions are:
    /// ```text
    /// STATE_START     -> STATE_IN_GRAPHEME  (on char accumulation)
    ///                 -> STATE_IN_ANSI      (on ANSI escape)
    /// STATE_IN_ANSI   -> STATE_START        (on ANSI terminator)
    /// STATE_IN_GRAPHEME -> STATE_IN_GRAPHEME (continuing cluster)
    ///                   -> STATE_START      (on boundary)
    /// ```
    ///
    /// The ASCII fast path optimizes single-character graphemes while maintaining
    /// proper boundary detection for sequences.
    ///
    /// # Arguments
    ///
    /// * `c` - The character to process
    ///
    /// # Returns
    ///
    /// * `Ok(Some(grapheme))` - A complete grapheme was formed
    /// * `Ok(None)` - Character was processed but no complete grapheme yet
    /// * `Err(error)` - An error occurred during processing:
    ///   - `GraphemeError::BufferOverflow` if cluster exceeds `MAX_GRAPHEME_SIZE`
    ///   - `GraphemeError::InvalidAnsiSequence` for malformed ANSI sequences
    #[inline]
    fn process_char(&mut self, c: char) -> Result<Option<Grapheme>> {
        let current_pos = self.position;
        self.position += c.len_utf8();

        match (c, self.state()) {
            // ASCII fast path - but only for definite boundaries
            (c, STATE_START) if c.is_ascii() && c != '\x1b' => {
                // Handle buffer state
                if self.buffer_len >= MAX_GRAPHEME_SIZE {
                    return Err(GraphemeError::buffer_overflow(current_pos, c.len_utf8()));
                }

                self.buffer[self.buffer_len] = c;
                self.buffer_len += 1;

                // Continue with normal boundary detection
                if self.buffer_len == 1 {
                    self.set_state(STATE_IN_GRAPHEME);
                    Ok(None)
                } else {
                    let grapheme = Grapheme::new(self.buffer, self.buffer_len - 1);
                    self.buffer[0] = self.buffer[self.buffer_len - 1];
                    self.buffer_len = 1;
                    self.set_state(STATE_IN_GRAPHEME);
                    Ok(Some(grapheme))
                }
            }
            ('\x1b', _) => {
                if self.buffer_len > 0 {
                    let grapheme = Grapheme::new(self.buffer, self.buffer_len);
                    self.buffer_len = 0;
                    self.set_state(STATE_IN_ANSI);
                    Ok(Some(grapheme))
                } else {
                    self.set_state(STATE_IN_ANSI);
                    Ok(None)
                }
            }
            (c, STATE_IN_ANSI) => {
                if c.is_ascii_alphabetic() {
                    self.set_state(STATE_START);
                    if self.count_ansi() {
                        self.buffer[0] = '\x1b';
                        Ok(Some(Grapheme::new(self.buffer, 1)))
                    } else {
                        Ok(None)
                    }
                } else if !c.is_ascii() {
                    Err(GraphemeError::invalid_ansi(current_pos, c.len_utf8()))
                } else {
                    Ok(None)
                }
            }
            (c, _) => {
                if self.buffer_len >= MAX_GRAPHEME_SIZE {
                    return Err(GraphemeError::buffer_overflow(current_pos, c.len_utf8()));
                }

                self.buffer[self.buffer_len] = c;
                self.buffer_len += 1;

                if self.buffer_len == 1 {
                    self.set_state(STATE_IN_GRAPHEME);
                    Ok(None)
                } else if c.is_ascii() || self.is_boundary(c) {
                    let grapheme = Grapheme::new(self.buffer, self.buffer_len - 1);
                    // Move the last character to the start of the buffer
                    self.buffer[0] = self.buffer[self.buffer_len - 1];
                    self.buffer_len = 1;
                    self.set_state(STATE_IN_GRAPHEME);
                    Ok(Some(grapheme))
                } else {
                    self.set_state(STATE_IN_GRAPHEME);
                    Ok(None)
                }
            }
        }
    }
}

impl<'a> Iterator for GraphemeIterator<'a> {
    type Item = Result<Grapheme>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = self.chars.next() {
            match self.process_char(c) {
                Ok(Some(grapheme)) => return Some(Ok(grapheme)),
                Ok(None) => continue,
                Err(e) => return Some(Err(e)),
            }
        }

        // Handle remaining buffer
        if self.buffer_len > 0 {
            let grapheme = Grapheme::new(self.buffer, self.buffer_len);
            self.buffer_len = 0;
            Some(Ok(grapheme))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::Vec;

    /// Maximum number of graphemes we'll test at once
    const TEST_VEC_SIZE: usize = 32;

    /// Helper to collect graphemes and check results
    fn collect_graphemes(input: &str, count_ansi: bool) -> Result<Vec<Grapheme, TEST_VEC_SIZE>> {
        let mut vec = Vec::new();
        let iter = GraphemeIterator::new(input, count_ansi);
        for result in iter {
            vec.extend_from_slice(&[result?])
                .map_err(|_| GraphemeError::buffer_overflow(0, 0))?;
        }
        Ok(vec)
    }

    #[test]
    fn test_ascii() {
        let text = "abc";
        let graphemes = collect_graphemes(text, false).unwrap();

        assert_eq!(graphemes.len(), 3);
        assert_eq!(graphemes[0].as_chars(), &['a']);
        assert_eq!(graphemes[1].as_chars(), &['b']);
        assert_eq!(graphemes[2].as_chars(), &['c']);
    }

    #[test]
    fn test_emoji() {
        let text = "👨‍👩‍👧‍👦";
        let graphemes = collect_graphemes(text, false).unwrap();

        assert_eq!(graphemes.len(), 1, "Family emoji should be one grapheme");
        let chars = graphemes[0].as_chars();
        assert!(chars.len() > 1, "Should contain multiple chars");
        assert_eq!(chars[0], '👨', "Should start with man emoji");
    }

    #[test]
    fn test_combining() {
        let text = "e\u{0301}"; // é decomposed
        let graphemes = collect_graphemes(text, false).unwrap();

        assert_eq!(
            graphemes.len(),
            1,
            "Combining sequence should be one grapheme"
        );
        assert_eq!(
            graphemes[0].len(),
            2,
            "Should contain base char and combining mark"
        );
        assert_eq!(graphemes[0].as_chars(), &['e', '\u{0301}']);
    }

    #[test]
    fn test_zwj_sequences() {
        // More comprehensive ZWJ tests
        let text = "👨‍💻";
        let graphemes = collect_graphemes(text, false).unwrap();
        assert_eq!(graphemes.len(), 1, "ZWJ sequence should be one grapheme");

        let text = "👨\u{200D}💻"; // Explicitly test ZWJ
        let graphemes = collect_graphemes(text, false).unwrap();
        assert_eq!(graphemes.len(), 1);
    }

    #[test]
    fn test_ansi() {
        let text = "\x1b[31mred\x1b[0m";
        let graphemes = collect_graphemes(text, true).unwrap();
        assert_eq!(graphemes.len(), 5); // 2 ANSI sequences + "red"
    }

    #[test]
    fn test_invalid_ansi() {
        let text = "\x1b\u{1234}"; // Invalid non-ASCII in ANSI sequence
        let result = collect_graphemes(text, true);
        assert!(matches!(
            result,
            Err(GraphemeError::InvalidAnsiSequence { .. })
        ));
    }

    #[test]
    fn test_buffer_overflow() {
        // Create string with too many combining marks
        let text = "a\u{0301}\u{0302}\u{0303}\u{0304}\u{0305}\u{0306}\u{0307}\u{0308}\u{0309}";
        let result = collect_graphemes(text, false);
        assert!(matches!(result, Err(GraphemeError::BufferOverflow { .. })));
    }

    // New test to verify heapless Vec capacity handling
    #[test]
    fn test_vec_capacity() {
        let text = "a".repeat(TEST_VEC_SIZE + 1);
        let result = collect_graphemes(&text, false);
        assert!(result.is_err(), "Should error on exceeding vec capacity");
    }
}
