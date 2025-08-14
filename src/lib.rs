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
#[macro_use]
mod macros;

use core::{
    cmp::PartialEq, 
    fmt, 
    ops::{Deref, DerefMut}, 
    ptr, 
    str::{from_utf8_unchecked, from_utf8_unchecked_mut}
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

impl<const CAP: usize> MicroStr<CAP>
{
    /* ##### STRUCT BUILDING ##### */
    
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
    /// Ok(MicroStr) - full size fits
    /// Err((MicroStr, usize)) - if only the first `usize` bytes were appended due to capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// match MicroStr::<5>::from_str("Hello, world!") {
    ///     Ok(string) => unreachable!(),
    ///     Err((string, fit_bytes)) => {
    ///         assert_eq!(string.as_str(), "Hello"); // truncated
    ///         assert_eq!(fit_bytes, 5);
    ///     }
    /// }
    /// ```
    #[must_use = "this returns a new `MicroStr`, it does not modify `self`"]
    pub fn from_str(s: &str) -> Result<Self, (Self, usize)> {
        let mut result = Self::new();
        match result.push_str(s) {
            Ok(()) => {Ok(result)},
            Err(bytes) => {Err((result, bytes))}
        }
    }

    /// Constructs a `MicroStr` from a string slice.
    /// 
    /// Equivalent [`MicroStr::from_str`] without Result returning and const support
    /// 
    /// If the input string is longer than the capacity, it is **truncated** to fit,
    /// ensuring UTF-8 validity (does not split multi-byte characters).
    /// 
    /// # Parameters
    /// - `s`: The input string slice
    /// 
    /// # Returns
    /// 
    /// `MicroStr`
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use microstr::*;
    /// let s = MicroStr::<5>::from_const("Hello, world!");
    /// assert_eq!(s.as_str(), "Hello"); // Truncated
    /// ```
    pub const fn from_const(s: &str) -> Self {
        let mut buffer = [0u8; CAP];
        let mut len = const_min(s.len(), CAP);
        let s_bytes = s.as_bytes();

        // SAFETY: Copy bytes minimal of str length and buffer size
        unsafe {
            ptr::copy_nonoverlapping(s.as_ptr(), buffer.as_mut_ptr(), const_min(s.len(), CAP));
        }

        // Character may be splitted
        // Checking: if last byte is continue-byte (10xxxxxx), go back.
        while len > 0
            && len < s_bytes.len()
            && len < CAP
            && (buffer[len - 1] & 0b1100_0000 == 0b1000_0000)
        {
            len -= 1;
        }

        // If character is splitted and last byte is begin of 2-, 3- or 4-byte
        // sequense and all bytes dont fit, go back too.
        if len > 0 && len < CAP && len < s_bytes.len() {
            let last_byte = buffer[len - 1];
            match last_byte {
                0b1100_0000..=0b1101_1111 => {
                    if len == CAP || len == s_bytes.len() {
                    } else if s_bytes[len] & 0b1100_0000 != 0b1000_0000 {
                        len -= 1;
                    }
                }
                0b1110_0000..=0b1110_1111 => {
                    if len + 1 >= CAP || len + 1 >= s_bytes.len() {
                        len -= 1;
                    } else if (s_bytes[len] & 0b1100_0000 != 0b1000_0000)
                        || (s_bytes[len + 1] & 0b1100_0000 != 0b1000_0000)
                    {
                        len -= 1;
                    }
                }
                0b1111_0000..=0b1111_0111 => {
                    if len + 2 >= CAP || len + 2 >= s_bytes.len() {
                        len -= 1;
                    } else if (s_bytes[len] & 0b1100_0000 != 0b1000_0000)
                        || (s_bytes[len + 1] & 0b1100_0000 != 0b1000_0000)
                        || (s_bytes[len + 2] & 0b1100_0000 != 0b1000_0000)
                    {
                        len -= 1;
                    }
                }
                _ => {}
            }
        }

        Self { buffer, len }
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

    /// Constructs a `MicroStr` from a string slice.
    /// 
    /// # Safety
    /// - s.len() must be less, than .capacity()
    /// 
    /// # Parameters
    /// 
    /// - `s`: The input string slice
    /// 
    /// # Returns
    /// 
    /// A new `MicroStr` containing up to `CAP` bytes of `s`.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use microstr::*;
    /// let s1 = unsafe { MicroStr::<16>::from_str_unchecked("Hello, world!") };
    /// // let s2 = unsafe { MicroStr::<15>::from_str_unchecked("Привет, мир!") }; // UB: 'м' be splitted
    /// ```
    pub const unsafe fn from_str_unchecked(s: &str) -> Self {
        let mut buf = [0; CAP];
        let to_copy = const_min(s.len(), CAP);
        ptr::copy_nonoverlapping(s.as_ptr(), buf.as_mut_ptr(), to_copy);
        Self {
            buffer: buf,
            len: to_copy
        }
    }

    /* ##### GETTERS ##### */

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
    #[inline]
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
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
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
    #[inline]
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// Returns the number of unused bytes available for writing.
    ///
    /// Equivalent to `self.capacity() - self.bytes_len()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let mut s = microstr!("Hi", 10);
    /// assert_eq!(s.extra_capacity(), 8);
    /// ```
    #[inline]
    pub fn extra_capacity(&self) -> usize {
        CAP - self.len
    }

    /// Returns `true` if the string has zero length.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let mut s: MicroStr<10> = MicroStr::new();
    /// assert!(s.is_empty());
    /// s.push('x');
    /// assert!(!s.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
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
    #[inline]
    pub fn bytes_len(&self) -> usize {
        self.len
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
        self.chars().count()
    }

    /* ##### PUSHERS ##### */

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
        let char_bytes = char_to_bytes_utf8(ch);
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
        ptr::copy_nonoverlapping(s.as_ptr(), self.as_mut_ptr().add(self.len), s.len());
        self.len += s.len();
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
    /// Ok(()) - full slice fits
    /// Err(usize) - if only the first `n` bytes were appended due to capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::MicroStr;
    /// let mut s = MicroStr::<6>::new();
    /// assert_eq!(s.push_str("An"), Ok(())); // An fits
    /// assert_eq!(s.push_str("河🌍"), Err(3)); // Only "河" fits (3 bytes), "🌍" excluded
    /// assert_eq!(s.as_str(), "An河");
    /// ```
    pub fn push_str(&mut self, s: &str) -> Result<(), usize> {
        let truncated = s.truncate(self.extra_capacity());
        let pushing_len = truncated.len();

        unsafe { self.push_str_unchecked(truncated); };

        if pushing_len == s.len() {
            return Ok(());
        }
        else {
            return Err(pushing_len);
        }
    }

    /* ##### TYPE CONVERTERS ##### */

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

    /// Returns a byte slice of the current content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s = microstr!("Hi", 10);
    /// assert_eq!(s.as_bytes(), b"Hi");
    /// ```
    #[inline]
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
    /// let mut s = microstr!("abc", 10);
    /// let bytes = s.as_mut_bytes();
    /// bytes[0] = b'x';
    /// assert_eq!(s.as_str(), "xbc");
    /// ```
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.buffer[..self.len]
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

    /* ##### MODIFICATORS ##### */

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
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
        self.buffer.get_mut(0).map(|x| *x = b'\0');
    }

    /// Truncates the string by index of **char**.
    ///
    /// If `char_idx` is greater than or equal to the number of characters,
    /// this is a no-op.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let mut s = microstr!("💖Rust", 10);
    /// s.truncate(1);
    /// assert_eq!(s.as_str(), "💖");
    /// ```
    pub fn truncate(&mut self, char_idx : usize) {
        if char_idx > self.len() { return; }
        let mut byte_idx = 0;
        for (idx, ch) in self.chars().enumerate() {
            if idx == char_idx {
                break;
            }
            byte_idx += ch.len_utf8();
        }
        // SAFETY:
        // - `byte_idx` is computed by summing `ch.len_utf8()` for valid UTF-8 characters.
        // - The loop stops when `idx == char_idx`, so `byte_idx` corresponds to the start of the next char.
        // - Since `char_idx < self.len()`, we know `byte_idx < self.len() <= CAP`.
        // - `self.as_mut_ptr()` is valid for `CAP` bytes.
        // - `byte_idx < CAP`, so `self.as_mut_ptr().add(byte_idx)` is in bounds.
        // - We write `0` (null terminator) — safe for UTF-8 and FFI.
        unsafe { self.as_mut_ptr().add(byte_idx).write(0) };
        self.len = byte_idx;
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

impl<const CAP: usize> fmt::Write for MicroStr<CAP> {
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push(c).map_err(|_| fmt::Error)
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        self.push_str(args.as_str().ok_or(fmt::Error)?).map_err(|_| fmt::Error)
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s).map_err(|_| fmt::Error)
    }
}

trait BoundaryCheckedTruncate {
    /// Truncates bytes to valid utf-8
    fn truncate<'a>(&'a self, i : usize) -> &'a Self;
}

impl BoundaryCheckedTruncate for str {
    fn truncate<'a>(&'a self, idx : usize) -> &'a Self {
        if idx >= self.len() { return self; }
        if idx == 0 { return ""; }

        let bytes = self.as_bytes();

        if !is_utf8_continuation(bytes[idx]) {
            // SAFETY: Slice already have valid UTF-8:
            // - first truncated byte isnt continue of multi-byte char
            // - &str already contains valid bytes
            return unsafe {from_utf8_unchecked(&bytes[..idx])}
        }

        let mut i = idx;

        while i > 0 {
            i -= 1;
            let current_byte = bytes[i];
            if !is_utf8_continuation(current_byte) {
                return unsafe { from_utf8_unchecked(&bytes[..i]) };
            }
            if i <= idx.saturating_sub(4) {
                // Max UTF-8 sequence is 4 bytes
                break;
            }
        }
        unsafe { from_utf8_unchecked(&bytes[..i]) }
    }
}


/// Returns `true` if the byte is a UTF-8 continuation byte (10xxxxxx)
#[inline(always)]
fn is_utf8_continuation(byte : u8) -> bool {
    byte & 0b1100_0000 == 0b1000_0000
}

/// const-fn analog to min
#[inline(always)]
const fn const_min(a : usize, b : usize) -> usize {
    if a <= b {
        a
    } else {
        b
    } 
}

/// Converts a Unicode character into its UTF-8 byte representation.
///
/// This is a helper method used internally to encode characters.
///
/// # Returns
///
/// A 4-byte array containing the UTF-8 encoding of `ch`, padded with zeros.
#[inline]
const fn char_to_bytes_utf8(ch: char) -> [u8; 4] {
    let mut result = [0; 4];
    ch.encode_utf8(&mut result);
    result
}
