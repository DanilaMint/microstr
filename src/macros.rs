// (M-RUST)

#[macro_export]
/// Creates a `MicroStr` containing the string slice.
/// 
/// `microstr!` allows optional argument `cap`
/// 
/// # Example
/// 
/// ```rust
/// use microstr::microstr;
/// let s_without_cap = microstr!("Hello, world");
/// let s_with_cap = microstr!("Hello, world", 20);
/// let s_with_less_cap = microstr!("Hello, world", 5);
/// 
/// assert_eq!(s_without_cap.capacity(), 12); // Minimal capacity to containing this string
/// assert_eq!(s_with_cap.capacity(), 20); // Capacity is set by user
/// assert_eq!(s_with_less_cap.capacity(), 5); // Capacity is set by user
/// assert_eq!(s_with_less_cap.as_str(), "Hello"); // Truncated
/// ```
macro_rules! microstr {
    ($s:expr) => {
        {
            static STR : &str = $s;
            unsafe { $crate::MicroStr::<{STR.len()}>::from_str_unchecked(STR) }
        }
    };
    ($s:expr, $cap:expr) => {
        {
            $crate::MicroStr::<{$cap}>::from_const($s)
        }
    };
}
