// (M-RUST)

use super::MicroStr;

impl<const CAP: usize> From<String> for MicroStr<CAP> {
    /// Converts a `String` into a `MicroStr`, truncating if necessary.
    ///
    /// # Note
    ///
    /// This method is provided for completeness, but prefer using [`MicroStr::from_str`]
    /// as `String` can be coerced to `&str`, and it's more explicit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let string = String::from("Hello world");
    /// let s: MicroStr<5> = MicroStr::from(string);
    /// assert_eq!(s.as_str(), "Hello");
    /// ```
    fn from(value: String) -> Self {
        match Self::from_str(&value) {
            Ok(s) => {s},
            Err((s, _)) => {s}
        }
    }
}

impl<const CAP: usize> From<MicroStr<CAP>> for String {
    /// Converts a `MicroStr` into a `String`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let stack_s = microstr!("Rust", 10);
    /// let string: String = String::from(stack_s);
    /// assert_eq!(string, "Rust");
    /// assert_eq!(string.capacity(), 10);
    /// ```
    fn from(value: MicroStr<CAP>) -> Self {
        let mut result = String::with_capacity(CAP);
        result.push_str(&value);
        result
    }
}
