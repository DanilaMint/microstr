use super::{MicroStr, microstr};

#[test]
fn basic_operations() {
    let mut s = MicroStr::<10>::new();
    assert_eq!(s.len(), 0);
    assert_eq!(s.capacity(), 10);
    assert_eq!(s.as_str(), "");

    assert_eq!(s.push('a'), Ok(()));
    assert_eq!(s.as_str(), "a");
    assert_eq!(s.len(), 1);
    assert_eq!(s.bytes_len(), 1);
}

#[test]
fn from_str_operations() {
    let s1 = MicroStr::<5>::from_str("hello");
    assert_eq!(s1.as_str(), "hello");

    let s2 = MicroStr::<3>::from_str("hello");
    assert_eq!(s2.as_str(), "hel");
}

#[test]
fn push_str_operations() {
    let mut s = MicroStr::<10>::new();
    assert_eq!(s.push_str("hello"), 5);
    assert_eq!(s.push_str(" world"), 5); // Only " worl" fits
    assert_eq!(s.as_str(), "hello worl");
}

#[test]
fn unsafe_operations() {
    let mut s = MicroStr::<10>::new();
    unsafe {
        s.push_unchecked('a');
        s.push_str_unchecked("bc");
    }
    assert_eq!(s.as_str(), "abc");
}

#[test]
fn raw_buffer_operations() {
    let mut buf = [0u8; 10];
    buf[..5].copy_from_slice(b"hello");
    let s = unsafe { MicroStr::from_raw_buffer(buf) };
    assert_eq!(s.as_str(), "hello");
    assert_eq!(s.into_raw_buffer(), buf);
}

#[test]
fn comparison_operations() {
    let s1 = MicroStr::<5>::from_str("hello");
    let s2 = MicroStr::<10>::from_str("hello");
    let s3 = MicroStr::<5>::from_str("world");
    
    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
}

#[test]
fn deref_operations() {
    let s = MicroStr::<10>::from_str("hello");
    assert_eq!(s.contains("ell"), true);
    assert_eq!(s.is_empty(), false);
}

#[test]
fn default_operations() {
    let s: MicroStr<10> = Default::default();
    assert_eq!(s.as_str(), "");
}

#[cfg(feature = "std")]
mod std_tests {
    use super::*;

    #[test]
    fn debug_display() {
        let s = MicroStr::<10>::from_str("test");
        assert_eq!(format!("{:?}", s), "MicroStr<10>{\"test\"}");
        assert_eq!(format!("{}", s), "test");
    }

    #[test]
    fn string_conversions() {
        let s = MicroStr::<10>::from_str("hello");
        let std_str: String = s.into();
        assert_eq!(std_str, "hello");

        let s2 = MicroStr::<10>::from(String::from("world"));
        assert_eq!(s2.as_str(), "world");
    }

    #[test]
    fn json_operations() {
        #[cfg(feature = "serde")]
        {
            let s = MicroStr::<20>::from_str("{\"key\":\"value\"}");
            let json = s.to_json().unwrap();
            assert_eq!(json, "\"{\\\"key\\\":\\\"value\\\"}\"");

            let parsed = MicroStr::<20>::from_json(&json).unwrap();
            assert_eq!(parsed.as_str(), "{\"key\":\"value\"}");
        }
    }
}

#[test]
fn utf8_edge_cases() {
    // 2-byte UTF-8
    let mut s = MicroStr::<3>::new();
    assert_eq!(s.push_str("ф"), 2); // Russian 'ф'
    assert_eq!(s.as_str(), "ф");

    // 3-byte UTF-8
    let mut s = MicroStr::<4>::new();
    assert_eq!(s.push_str("€"), 3); // Euro sign
    assert_eq!(s.as_str(), "€");

    // 4-byte UTF-8
    let mut s = MicroStr::<5>::new();
    assert_eq!(s.push_str("😊"), 4); // Emoji
    assert_eq!(s.as_str(), "😊");

    // Partial UTF-8
    let mut s = MicroStr::<3>::new();
    assert_eq!(s.push_str("😊"), 0); // Not enough space for 4-byte emoji
    assert_eq!(s.as_str(), "");
}

#[test]
fn capacity_edge_cases() {
    // Exact capacity
    let s = MicroStr::<5>::from_str("hello");
    assert_eq!(s.as_str(), "hello");

    // Zero capacity
    let mut s = MicroStr::<0>::new();
    assert_eq!(s.push('a'), Err(()));
    assert_eq!(s.push_str("test"), 0);
}

#[test]
fn macro_tests() {
    let s1 = microstr!("Hello, world");
    let s2 = microstr!("Hello, world", 20);
    let s3 = microstr!("Hello, world", 5);

    assert_eq!(s1.capacity(), 12);
    assert_eq!(s2.capacity(), 20);
    assert_eq!(s3.capacity(), 5);
}