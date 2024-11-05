//! Unicode grapheme cluster handling and boundary detection.
//!
//! This module implements grapheme cluster boundary detection according to Unicode Standard
//! Annex #29 (UAX #29). It handles:
//! - Emoji sequences and modifiers
//! - Combining marks and accents
//! - Zero Width Joiners (ZWJ)
//! - Regional indicators (flags)
//! - Right-to-Left (RTL) text
//!
//! The implementation uses bit patterns for efficient boundary detection and fixed-size
//! buffers to maintain zero allocation guarantees.

use crate::MAX_GRAPHEME_SIZE;

/// Unicode grapheme cluster boundary detection rules encoded as bit patterns.
///
/// These constants represent different character categories that affect grapheme
/// cluster boundaries according to UAX #29. Each category is assigned a unique
/// bit flag for efficient boundary detection through bitwise operations.
pub mod boundary {
    /// Extending marks that don't create boundaries (e.g., combining accents)
    pub const EXTEND: u32 = 0x01;
    /// Zero Width Joiner - used in emoji sequences
    pub const ZWJ: u32 = 0x02;
    /// Spacing marks that modify the base character
    pub const SPACINGMARK: u32 = 0x04;
    /// Characters that don't create boundaries with following characters
    pub const PREPEND: u32 = 0x08;
    /// Regional indicator symbols used in flag emoji
    pub const REGIONAL: u32 = 0x10;
    /// Emoji modifiers like skin tones
    pub const EMOJI_MOD: u32 = 0x20;
}

/// A fixed-size grapheme cluster representation.
///
/// Stores a sequence of Unicode characters that form a single grapheme cluster.
/// The size is limited to [`MAX_GRAPHEME_SIZE`] code points to maintain
/// zero-allocation guarantees while handling complex emoji sequences.
///
/// # Examples
///
/// ```
/// use graphmemes::Grapheme;
///
/// let chars = ['\u{0061}', '\u{0301}', '\0', '\0', '\0', '\0', '\0', '\0'];
/// let grapheme = Grapheme::new(chars, 2); // "á" (a + combining acute)
/// assert_eq!(grapheme.len(), 2);
/// assert_eq!(grapheme.as_chars(), &['\u{0061}', '\u{0301}']);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Grapheme {
    /// Fixed-size array of characters in the cluster
    chars: [char; MAX_GRAPHEME_SIZE],
    /// Number of valid characters in the array
    len: usize,
}

impl Grapheme {
    /// Creates a new grapheme cluster from a fixed-size character array.
    ///
    /// # Arguments
    ///
    /// * `chars` - Fixed-size array of characters
    /// * `len` - Number of valid characters in the array
    ///
    /// # Examples
    ///
    /// ```
    /// use graphmemes::Grapheme;
    ///
    /// let chars = ['a', '\0', '\0', '\0', '\0', '\0', '\0', '\0'];
    /// let grapheme = Grapheme::new(chars, 1);
    /// assert_eq!(grapheme.as_chars(), &['a']);
    /// ```
    #[inline]
    pub fn new(chars: [char; MAX_GRAPHEME_SIZE], len: usize) -> Self {
        Self { chars, len }
    }

    /// Returns a slice of the valid characters in this grapheme cluster.
    ///
    /// # Examples
    ///
    /// ```
    /// use graphmemes::Grapheme;
    ///
    /// let chars = ['é', '\0', '\0', '\0', '\0', '\0', '\0', '\0'];
    /// let grapheme = Grapheme::new(chars, 1);
    /// assert_eq!(grapheme.as_chars(), &['é']);
    /// ```
    #[inline]
    pub fn as_chars(&self) -> &[char] {
        &self.chars[..self.len]
    }

    /// Returns the number of characters in this grapheme cluster.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if this grapheme cluster contains no characters.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Determines the boundary category of a character for grapheme clustering.
    ///
    /// This function categorizes characters according to UAX #29 rules using
    /// efficient bit patterns for boundary detection.
    ///
    /// # Arguments
    ///
    /// * `c` - The character to categorize
    ///
    /// # Returns
    ///
    /// A bit pattern indicating the character's category, where zero means
    /// the character forms a boundary.
    #[inline]
    pub(crate) fn char_category(c: char) -> u32 {
        use boundary::*;
        match c {
            '\u{200D}' => ZWJ,
            '\u{FE0F}' => EMOJI_MOD,
            '\u{1F3FB}'..='\u{1F3FF}' => EMOJI_MOD,
            '\u{1F1E6}'..='\u{1F1FF}' => REGIONAL,
            c if c.is_ascii() => 0,
            c if is_extend(c) => EXTEND,
            c if is_spacing_mark(c) => SPACINGMARK,
            c if is_prepend(c) => PREPEND,
            _ => 0,
        }
    }
}

/// Determines if a character is an emoji.
///
/// Checks if the character falls within the Unicode ranges designated for emoji
/// symbols, including basic emoji, symbol emoji, and dingbats.
#[inline]
pub(crate) fn is_emoji(c: char) -> bool {
    matches!(c,
        '\u{1F300}'..='\u{1F9FF}' | // Emoji
        '\u{2600}'..='\u{26FF}' |   // Misc symbols
        '\u{2700}'..='\u{27BF}'     // Dingbats
    )
}

/// Determines if a character is an extending mark.
///
/// Checks if the character is a combining mark that should not create a new
/// grapheme cluster boundary, including various types of diacritical marks
/// and combining characters.
#[inline]
fn is_extend(c: char) -> bool {
    matches!(c,
        '\u{0300}'..='\u{036F}' |  // Combining marks
        '\u{1AB0}'..='\u{1AFF}' |  // Extended combining marks
        '\u{1DC0}'..='\u{1DFF}' |  // Supplement combining marks
        '\u{20D0}'..='\u{20FF}' |  // Combining marks for symbols
        '\u{FE20}'..='\u{FE2F}'    // Combining half marks
    )
}

/// Determines if a character is a spacing mark.
///
/// Checks if the character is a spacing mark in various scripts that
/// should not create a new grapheme cluster boundary.
#[inline]
fn is_spacing_mark(c: char) -> bool {
    matches!(c,
        '\u{0903}' |       // Devanagari Sign Visarga
        '\u{093E}'..='\u{0940}' | // Devanagari vowel signs
        '\u{0949}'..='\u{094C}' | // More Devanagari signs
        '\u{094E}'..='\u{094F}' | // Final Devanagari signs
        '\u{0982}'..='\u{0983}'   // Bengali Sign Visarga
    )
}

/// Determines if a character is a prepend character.
///
/// Checks if the character is one that should be prepended to the following
/// characters without creating a grapheme cluster boundary.
#[inline]
fn is_prepend(c: char) -> bool {
    matches!(c,
        '\u{0600}'..='\u{0605}' |  // Arabic numbers
        '\u{06DD}' |               // Arabic End Of Ayah
        '\u{070F}' |               // Syriac Abbreviation Mark
        '\u{0890}'..='\u{0891}' |  // Arabic Tone marks
        '\u{08E2}'                 // Arabic Disputed End Of Ayah
    )
}
