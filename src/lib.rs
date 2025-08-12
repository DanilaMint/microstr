#![cfg_attr(not(feature = "std"), no_std)]
//! # MicroStr — Fixed-capacity stack-allocated string
//!
//! A lightweight, stack-allocated string type with fixed capacity and UTF-8 support.
//! Designed to work in both `std` and `no_std` environments.
//!
//! ## Features
//!
//! - **No heap allocations**: All data is stored on the stack.
//! - **UTF-8 safe**: Guarantees valid UTF-8 content.
//! - **Fixed capacity**: Determined at compile time via const generic `CAP`.
//! - **`no_std` by default**: Optional `std` and `serde` support via Cargo features.
//!
//! ## Cargo Features
//!
//! - `std` *(optional)*: Enables `Display`, `Debug`, `From<String>`, and other std traits.
//! - `serde` *(optional, requires `std`)*: Enables JSON serialization/deserialization.
//!
//! ## Example
//!
//! ```rust
//! use microstr::*;
//!
//! let mut s: MicroStr<16> = MicroStr::new();
//! s.push_str("Hello");
//! s.push('!');
//! assert_eq!(s.as_str(), "Hello!");
//! ```

#[cfg(test)]
mod tests;

#[cfg(feature = "std")]
mod std_only;

mod macros;

#[cfg(feature = "std")]
pub use std_only::*;
use core::{
    ptr,
    str::{from_utf8_unchecked, from_utf8_unchecked_mut},
    cmp::PartialEq,
    ops::{Deref, DerefMut},
};

/// A fixed-capacity, stack-allocated string with UTF-8 support.
///
/// `MicroStr<CAP>` stores up to `CAP` bytes of UTF-8 data directly on the stack.
/// It does not perform heap allocation, making it ideal for `no_std` environments
/// and performance-critical code.
///
/// # Usage
///
/// ```rust
/// use microstr::*;
/// let mut s: MicroStr<32> = MicroStr::new();
/// s.push_str("Rust");
/// s.push('!');
/// assert_eq!(s.as_str(), "Rust!");
/// ```
///
/// # Capacity and Truncation
///
/// If you attempt to add more data than the capacity allows, the input is **truncated**
/// to fit, ensuring no overflow and maintaining UTF-8 validity.
///
/// # Type Parameters
///
/// - `CAP`: The maximum number of bytes this string can hold (must be at least 1).
///
/// # Notes
///
/// - The internal buffer is always valid UTF-8.
/// - Methods like `push_str` ensure partial UTF-8 sequences are not split.
#[derive(Clone)]
pub struct MicroStr<const CAP: usize> {
    buffer: [u8; CAP],
    len: usize,
}

impl<const CAP: usize> MicroStr<CAP> {
    /// Converts a Unicode character into its UTF-8 byte representation.
    ///
    /// This is a helper method used internally to encode characters.
    ///
    /// # Returns
    ///
    /// A 4-byte array containing the UTF-8 encoding of `ch`, padded with zeros.
    const fn char_to_bytes_utf8(ch: char) -> [u8; 4] {
        let mut result = [0; 4];
        ch.encode_utf8(&mut result);
        result
    }

    /// Creates an empty `MicroStr`.
    ///
    /// The string has length 0 and can hold up to `CAP` bytes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s: MicroStr<10> = MicroStr::new();
    /// assert_eq!(s.len(), 0);
    /// assert_eq!(s.capacity(), 10);
    /// ```
    pub const fn new() -> Self {
        Self {
            buffer: [0; CAP],
            len: 0,
        }
    }

    /// Constructs a `MicroStr` from a string slice.
    ///
    /// If the input string is longer than the capacity, it is **truncated** to fit,
    /// ensuring UTF-8 validity (does not split multi-byte characters).
    ///
    /// # Parameters
    ///
    /// - `s`: The input string slice.
    ///
    /// # Returns
    ///
    /// A new `MicroStr` containing up to `CAP` bytes of `s`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s: MicroStr<5> = MicroStr::from_str("Hello, world!");
    /// assert_eq!(s.as_str(), "Hello"); // Truncated
    /// ```
    pub fn from_str(s: &str) -> Self {
        let mut result = Self::new();
        result.push_str(s);
        result
    }

    /// Constructs a `MicroStr` from a raw byte buffer.
    ///
    /// The string length is determined by the first null byte (`0x00`) in the buffer.
    /// The bytes before the null terminator must form a valid UTF-8 sequence.
    ///
    /// # Safety
    ///
    /// - The buffer must contain valid UTF-8 data up to the first `0x00` byte.
    /// - The buffer must be exactly `CAP` bytes long.
    ///
    /// # Example (unsafe)
    ///
    /// ```rust
    /// use microstr::*;
    /// let buf = *b"Hello\0\0\0\0\0";
    /// let s = unsafe { MicroStr::<10>::from_raw_buffer(buf) };
    /// assert_eq!(s.as_str(), "Hello");
    /// ```
    pub unsafe fn from_raw_buffer(buf: [u8; CAP]) -> Self {
        let mut len = 0;
        for byte in &buf {
            if *byte == 0 {
                break;
            }
            len += 1;
        }
        Self { buffer: buf, len }
    }

    /// Appends a character to the end of the string without bounds checking.
    ///
    /// # Safety
    ///
    /// - The UTF-8 byte length of `ch` plus the current length of the string
    ///   must be **less than or equal to** `CAP`. Otherwise, buffer overflow occurs.
    ///
    /// # Example (unsafe)
    ///
    /// ```rust
    /// use microstr::*;
    /// let mut s: MicroStr<10> = MicroStr::new();
    /// unsafe { s.push_unchecked('A') };
    /// assert_eq!(s.as_str(), "A");
    /// ```
    pub unsafe fn push_unchecked(&mut self, ch: char) {
        let char_len = ch.len_utf8();
        let char_bytes = Self::char_to_bytes_utf8(ch);
        let char_ptr = char_bytes.as_ptr();
        let buf_ptr = self.as_mut_ptr().add(self.len);
        ptr::copy_nonoverlapping(char_ptr, buf_ptr, char_len);
        self.len += char_len;
    }

    /// Appends a character to the end of the string.
    ///
    /// # Parameters
    ///
    /// - `ch`: The character to append.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the character was successfully added.
    /// - `Err(())` if there is insufficient space (including UTF-8 byte length).
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let mut s = MicroStr::<1>::new();
    /// assert!(s.push('A').is_ok());
    /// assert!(s.push('B').is_err()); // No space
    /// ```
    pub fn push(&mut self, ch: char) -> Result<(), ()> {
        if ch.len_utf8() + self.len <= CAP {
            // SAFETY: checked length
            unsafe { self.push_unchecked(ch) };
            return Ok(());
        }
        Err(())
    }

    /// Appends raw bytes to the string without UTF-8 or bounds checking.
    ///
    /// # Safety
    ///
    /// - `bytes` must be valid UTF-8.
    /// - `self.len() + bytes.len()` must not exceed `CAP`.
    ///
    /// This is a low-level helper method; prefer `push_str` or `push_str_unchecked`.
    unsafe fn push_bytes(&mut self, bytes: &[u8]) {
        self.buffer[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
    }

    /// Appends a string slice without bounds checking.
    ///
    /// # Safety
    ///
    /// - The byte length of `s` plus the current length must be ≤ `CAP`.
    /// - `s` must be valid UTF-8.
    ///
    /// # Example (unsafe)
    ///
    /// ```rust
    /// use microstr::microstr;
    /// let mut s = microstr!("", 5);
    /// unsafe { s.push_str_unchecked("Hi") };
    /// assert_eq!(s.as_str(), "Hi");
    /// ```
    pub unsafe fn push_str_unchecked(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.push_bytes(bytes);
    }

    /// Appends a string slice, truncating if necessary to fit capacity.
    ///
    /// Ensures UTF-8 validity by not splitting multi-byte characters.
    ///
    /// # Parameters
    ///
    /// - `s`: The string slice to append.
    ///
    /// # Returns
    ///
    /// The number of **bytes** actually appended.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::MicroStr;
    /// let mut s = MicroStr::<6>::new();
    /// let appended = s.push_str("An河🌍"); // "An河🌍" is 9 bytes
    /// assert_eq!(appended, 5); // Only "An河" fits (5 bytes), "🌍" excluded
    /// assert_eq!(s.as_str(), "An河");
    /// ```
    pub fn push_str(&mut self, s: &str) -> usize {
        let available = CAP - self.len;
        if available == 0 {
            return 0;
        }

        let str_bytes = s.as_bytes();
        let max_possible_copy = str_bytes.len().min(available);

        // Find the largest valid UTF-8 prefix that fits
        let mut valid_len = 0;
        let mut i = 0;
        while i < max_possible_copy {
            let byte = str_bytes[i];
            let char_len = match byte {
                0x00..=0x7F => 1,
                0xC0..=0xDF => 2,
                0xE0..=0xEF => 3,
                0xF0..=0xFF => 4,
                _ => 1, // continuation byte (already handled)
            };

            if i + char_len > max_possible_copy {
                break;
            }
            valid_len = i + char_len;
            i = valid_len;
        }

        if valid_len > 0 {
            unsafe {
                self.push_bytes(&str_bytes[..valid_len]);
            }
        }

        valid_len
    }

    /// Returns a string slice of the current content.
    ///
    /// This slice is guaranteed to be valid UTF-8.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s = microstr!("Hello", 10);
    /// assert_eq!(s.as_str(), "Hello");
    /// ```
    pub fn as_str(&self) -> &str {
        // SAFETY: buffer always contains valid UTF-8
        unsafe { from_utf8_unchecked(self.as_bytes()) }
    }

    /// Returns a mutable string slice of the current content.
    ///
    /// Allows in-place mutation of the string, but you must ensure the result remains valid UTF-8.
    ///
    /// # Safety
    ///
    /// The caller must ensure that any modifications preserve UTF-8 validity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let mut s = microstr!("Hello", 10);
    /// let s_mut = s.as_str_mut();
    /// s_mut.make_ascii_uppercase();
    /// assert_eq!(s.as_str(), "HELLO");
    /// ```
    pub fn as_str_mut(&mut self) -> &mut str {
        // SAFETY: buffer always contains valid UTF-8
        unsafe { from_utf8_unchecked_mut(self.as_mut_bytes()) }
    }

    /// Returns a raw pointer to the first byte of the internal buffer.
    ///
    /// Useful for FFI or low-level operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s = microstr!("Hi", 10);
    /// let ptr = s.as_ptr();
    /// assert_eq!(unsafe { *ptr }, b'H');
    /// ```
    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    /// Returns a mutable raw pointer to the first byte of the internal buffer.
    ///
    /// Useful for FFI or zero-copy input parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let mut s = MicroStr::<10>::new();
    /// let ptr = s.as_mut_ptr();
    /// unsafe {
    ///     *ptr = b'X';
    /// }
    /// ```
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    /// Returns a byte slice of the current content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s = microstr!("Hi", 10);
    /// assert_eq!(s.as_bytes(), b"Hi");
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[..self.len]
    }

    /// Returns a mutable byte slice of the current content.
    ///
    /// You must ensure that any modifications result in valid UTF-8.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let mut s = MicroStr::<10>::from_str("abc");
    /// let bytes = s.as_mut_bytes();
    /// bytes[0] = b'x';
    /// assert_eq!(s.as_str(), "xbc");
    /// ```
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.buffer[..self.len]
    }

    /// Returns the number of bytes currently used in the string.
    ///
    /// This is the length in bytes, not Unicode scalar values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s = microstr!("💖", 10);
    /// assert_eq!(s.bytes_len(), 4); // 4-byte UTF-8 emoji
    /// ```
    pub fn bytes_len(&self) -> usize {
        self.len
    }

    /// Returns the total capacity in bytes.
    ///
    /// This is the maximum number of bytes the string can hold.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s: MicroStr<32> = MicroStr::new();
    /// assert_eq!(s.capacity(), 32);
    /// ```
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// Returns the number of Unicode scalar values (chars) in the string.
    ///
    /// This is computed by iterating over `chars()`, so it's O(n).
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s = microstr!("💖Rust", 10);
    /// assert_eq!(s.len(), 5); // '💖' is one char, 'R','u','s','t'
    /// ```
    pub fn len(&self) -> usize {
        self.as_str().chars().count()
    }

    /// Consumes the `MicroStr` and returns the raw byte buffer.
    ///
    /// The buffer is exactly `CAP` bytes long. Unused bytes are unspecified.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s = microstr!("Hi", 8);
    /// let buf = s.into_raw_buffer();
    /// assert_eq!(&buf[..2], b"Hi");
    /// ```
    pub fn into_raw_buffer(self) -> [u8; CAP] {
        self.buffer
    }

    /// Clears str to `default` state.
    /// 
    /// Sets length as 0 and first byte b'\0'
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use microstr::*;
    /// let mut s = microstr!("Clear me!");
    /// s.clear();
    /// assert_eq!(s.as_str(), "");
    /// ```
    pub fn clear(&mut self) {
        self.len = 0;
        self.buffer.get_mut(0).map(|x| *x = 0);
    }
}

impl<const CAP: usize> Default for MicroStr<CAP> {
    /// Returns an empty `MicroStr`.
    ///
    /// Equivalent to [`MicroStr::new()`].
    fn default() -> Self {
        Self::new()
    }
}

impl<const A: usize, const B: usize> PartialEq<MicroStr<B>> for MicroStr<A> {
    /// Compares two `MicroStr`s for equality by content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let a = microstr!("test", 10);
    /// let b = microstr!("test", 15);
    /// assert_eq!(a, b);
    /// ```
    fn eq(&self, other: &MicroStr<B>) -> bool {
        self.as_str() == other.as_str()
    }

    /// Compares two `MicroStr`s for inequality by content.
    fn ne(&self, other: &MicroStr<B>) -> bool {
        self.as_str() != other.as_str()
    }
}

impl<const CAP: usize> Deref for MicroStr<CAP> {
    type Target = str;

    /// Allows `MicroStr` to be used like a `&str`.
    ///
    /// Enables calling string methods directly:
    ///
    /// ```rust
    /// use microstr::*;
    /// let s = microstr!("hello");
    /// assert!(s.starts_with("he"));
    /// assert_eq!(s.to_uppercase(), "HELLO");
    /// ```
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const CAP: usize> DerefMut for MicroStr<CAP> {
    /// Allows mutable access to the string content via `&mut str`.
    ///
    /// Enables in-place string modification.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let mut s = microstr!("rust");
    /// s.make_ascii_uppercase();
    /// assert_eq!(s.as_str(), "RUST");
    /// ```
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_str_mut()
    }
}