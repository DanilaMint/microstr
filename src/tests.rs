use core::fmt::Write;

use crate::utf8_truncator;

use super::{MicroStr, microstr};

/* BASE METHODS */
#[test]
fn new() {
    let s: MicroStr<10> = MicroStr::new();
    assert_eq!(s.as_str(), "");
    assert_eq!(s.len(), 0);
}

#[test]
fn from_str() {
    let s = MicroStr::<15>::from_str("Hello, world").expect("Unreachable");
    assert_eq!(s.as_str(), "Hello, world");
    
    let (s, bytes) = MicroStr::<15>::from_str("Привет, мир").unwrap_err();
    assert_eq!(s.as_str(), "Привет, "); // 'м' has been splitted
    assert_eq!(bytes, 14);
}

#[test]
fn from_const() {
    let s = MicroStr::<15>::from_const("Constant");
    assert_eq!(s.as_str(), "Constant");
}

#[test]
fn from_raw_buffer() {
    let buffer = [b'R', b'a', b'w'];
    let s = unsafe { MicroStr::<8>::from_raw_buffer(buffer) };
    assert_eq!(s.as_str(), "Raw");
}

#[test]
fn from_str_unchecked() {
    let s = unsafe { MicroStr::<15>::from_str_unchecked("Hello, world") };
    assert_eq!(s.as_str(), "Hello, world");
}

#[test]
fn pointers() {
    let mut s = microstr!("Hello, world!");

    unsafe {
        assert_eq!(*s.as_ptr(), b'H');
        assert_eq!(*s.as_ptr().add(4), b'o');

        *s.as_mut_ptr().add(4) = b',';
        *s.as_mut_ptr().add(5) = b' ';
        *s.as_mut_ptr().add(6) = b'u';
        *s.as_mut_ptr().add(7) = b'n';
        *s.as_mut_ptr().add(8) = b's';
        *s.as_mut_ptr().add(9) = b'a';
        *s.as_mut_ptr().add(10) = b'f';
        *s.as_mut_ptr().add(11) = b'e';
    }
    assert_eq!(s.as_str(), "Hell, unsafe!");
}

#[test]
fn constants_and_variables() {
    let s = microstr!("Кот", 10);

    assert_eq!(s.capacity(), 10);
    assert_eq!(s.len(), 3);
    assert_eq!(s.bytes_len(), 6);
    assert_eq!(s.extra_capacity(), 4);
    assert!(!s.is_empty());

    let s = MicroStr::<10>::new();
    assert!(s.is_empty());
}

#[test]
fn push_char() {
    let mut s = MicroStr::<6>::new();

    assert_eq!(s.push('a'), Ok(()));
    assert_eq!(s.push('👿'), Ok(()));
    assert_eq!(s.push('ш'), Err(()));
    assert_eq!(s.as_str(), "a👿");
    
    let mut s = MicroStr::<4>::new();
    unsafe {
        s.push_unchecked('🦀');
    }
    assert_eq!(s.as_str(), "🦀");
}

#[test]
fn push_str() {
    let mut s = microstr!("Hello, ", 15);
    assert_eq!(s.push_str("world!"), Ok(()));
    assert_eq!(s.as_str(), "Hello, world!");
    assert_eq!(s.push_str(" NOT FIT"), Err(2));
    assert_eq!(s.as_str(), "Hello, world! N");
}

#[test]
fn bytes() {
    let mut s = microstr!("Rust?", 10);
    assert_eq!(s.as_bytes(), &[b'R', b'u', b's', b't', b'?'][..]);
    s.as_mut_bytes()[4] = b'!';
    assert_eq!(s.as_str(), "Rust!");
}

#[test]
fn into_raw_buffer() {
    let s = microstr!("RAW", 4);
    let buf = s.into_raw_buffer();

    assert_eq!(buf, [b'R', b'A', b'W', 0]);
}

#[test]
fn clear() {
    let mut s = microstr!("Dαηίlα Mίητ");
    s.clear();
    assert_eq!(s.as_str(), "");
    assert_eq!(s.len(), 0);
}

#[test]
fn truncate() {
    let mut s = microstr!("Номер 1234567890");
    s.truncate(11);
    assert_eq!(s.as_str(), "Номер 12345");
}

#[test]
fn default() {
    let s: MicroStr<10> = MicroStr::default();
    assert_eq!(s.as_str(), "");
    assert_eq!(s.len(), 0);
}

#[test]
fn compare() {
    let s1 = microstr!("hello", 5);
    let s2 = microstr!("hello", 10);
    let s3 = microstr!("world", 5);

    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
    assert_ne!(s2, s3);
}

#[test]
fn deref() {
    let s = microstr!("Hello", 15);

    assert!( s.is_ascii() );
    assert_eq!(s.to_ascii_uppercase(), "HELLO");
}

#[test]
fn fmt() {
    let mut s = microstr!("", 50);
    assert_eq!(s.write_char('a'), Ok(()));
    assert_eq!(s.write_str("bcdef"), Ok(()));
    assert_eq!(s.write_fmt(format_args!("; {} = {}", "var", 10)), Ok(()));

    assert_eq!(s.as_str(), "abcdef; var = 10");
}

#[test]
fn truncator() {
    let s = "Hello, world";
    assert_eq!(utf8_truncator(s, 0), 0);    // ""
    assert_eq!(utf8_truncator(s, 20), 12);  // "Hello, world"
    assert_eq!(utf8_truncator(s, 10), 10);  // "Hello, wor"

    let s = "Привет, мир";
    assert_eq!(utf8_truncator(s, 10), 10);  // "Приве"
    assert_eq!(utf8_truncator(s, 11), 10);  // "Приве"
    assert_eq!(utf8_truncator(s, 12), 12);  // "Привет"
    assert_eq!(utf8_truncator(s, 13), 13);  // "Привет,"

    let s = "你好，世界";
    assert_eq!(utf8_truncator(s, 3), 3);  // "你"
    assert_eq!(utf8_truncator(s, 4), 3);  // "你"
    assert_eq!(utf8_truncator(s, 5), 3);  // "你"
    assert_eq!(utf8_truncator(s, 6), 6);  // "你好"

    let s = "🔥🦀❗️";
    assert_eq!(utf8_truncator(s, 3), 0);  // ""
    assert_eq!(utf8_truncator(s, 4), 4);  // "🔥"
    assert_eq!(utf8_truncator(s, 5), 4);  // "🔥"
    assert_eq!(utf8_truncator(s, 6), 4);  // "🔥"
    assert_eq!(utf8_truncator(s, 7), 4);  // "🔥"
    assert_eq!(utf8_truncator(s, 8), 8);  // "🔥🦀"
}

/* STD ONLY */

#[test]
fn output() {
    let s = microstr!("Some Output", 25);
    assert_eq!(format!("{:?}", s), "MicroStr<25>{\"Some Output\"}");
    assert_eq!(format!("{}", s), "Some Output");
}

#[test]
fn string() {
    let string = String::from("Heap Allocated!");

    let s = MicroStr::<20>::from(string);

    assert_eq!(s.as_str(), "Heap Allocated!");

    let return_string = String::from(s);

    assert_eq!(return_string, "Heap Allocated!");
}

#[test]
#[cfg(feature = "serde")]
fn serde() {
    let string = microstr!("{\"key\": 42}");
    string.to_json();
}