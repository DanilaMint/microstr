# MicroStr

[![crates.io](https://img.shields.io/crates/v/microstr.svg)](https://crates.io/crates/microstr)  
[![Documentation](https://docs.rs/microstr/badge.svg)](https://docs.rs/microstr)  
[![License](https://img.shields.io/crates/l/microstr)](https://github.com/DanilaMint/microstr/blob/main/LICENSE)

A lightweight, stack-allocated string with fixed capacity and UTF-8 support.  
Ideal for `no_std` environments, embedded systems, and performance-critical code.

```toml
[dependencies]
microstr = "0.1"
```

## Features

- ✅ **No heap allocations** — fully stack-based.  
- ✅ **UTF-8 safe** — always valid string content.  
- ✅ **`no_std` by default** — works without `std`.  
- ✅ **Const generic capacity** — size known at compile time.  
- ✅ **Truncation-aware** — safely appends strings without overflow.  
- ✅ **Seamless `std` integration** — supports `Display`, `Debug`, `From<String>`, etc.  
- ✅ **Optional `serde` support** — JSON (de)serialization with length checking.  

## Usage

```rust
use microstr::MicroStr;

// Create a string with capacity of 16 bytes
let mut s: MicroStr<16> = MicroStr::new();

s.push_str("Hello");
s.push('!');

assert_eq!(s.as_str(), "Hello!");
assert_eq!(s.len(), 6);           // 6 Unicode chars
assert_eq!(s.bytes_len(), 6);     // 6 bytes
assert!(s.push_str(" this won't fit entirely") > 0); // Truncated safely
```

You can also use it like a regular `&str` thanks to `Deref`:

```rust
if s.starts_with("Hello") {
    println!("Greeting: {}", s);
}
```

## Features

Enable optional features in `Cargo.toml`:

```toml
[dependencies]
microstr = { version = "0.1", features = ["std", "serde"] }
```

| Feature | Description |
|--------|-------------|
| `std` (default: on) | Enables `Display`, `Debug`, `From<String>`, and `ToString`. Required for `serde`. |
| `serde` | Enables `.to_json()` and `StackString::from_json()` with capacity checks. |

### Example with `serde`

```rust
#[cfg(feature = "serde")]
{
    let s = MicroStr::from_str::<10>("hello");
    let json = s.to_json().unwrap();
    assert_eq!(json, r#""hello""#);

    let s2 = MicroStr::from_json(r#""world""#).unwrap();
    assert_eq!(s2, "world");
}
```

## Why MicroStr?

- **Predictable performance**: No allocations, no heap usage.  
- **Memory safety**: Always valid UTF-8, no buffer overflows.  
- **Great for constrained environments**: Embedded, kernels, WASM.  
- **Easy migration**: Drop-in replacement for `String` in many cases.  

## API Documentation

📚 Full documentation: [https://docs.rs/microstr](https://docs.rs/microstr)

## License

[MIT License](./LICENSE)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.