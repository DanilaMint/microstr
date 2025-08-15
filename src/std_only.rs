use std::fmt;
use core::fmt::Formatter;
use super::MicroStr;

impl<const CAP: usize> fmt::Debug for MicroStr<CAP> {
    /// Formats the `MicroStr` for debugging.
    ///
    /// Output format: `MicroStr<{CAP}>"{content}"`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s = microstr!("test", 10);
    /// assert_eq!(format!("{:?}", s), "MicroStr<10>{\"test\"}");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MicroStr<{}>{{\"{}\"}}", CAP, self.as_str())
    }
}

impl<const CAP: usize> fmt::Display for MicroStr<CAP> {
    /// Formats the `MicroStr` as a regular string.
    ///
    /// Useful for printing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microstr::*;
    /// let s = microstr!("Hello", 10);
    /// assert_eq!(format!("{}", s), "Hello");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

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
