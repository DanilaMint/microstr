# MicroStr

[![crates.io](https://img.shields.io/crates/v/microstr.svg)](https://crates.io/crates/microstr)  
[![Documentation](https://docs.rs/microstr/badge.svg)](https://docs.rs/microstr)  
[![License](https://img.shields.io/crates/l/microstr)](https://github.com/DanilaMint/microstr/blob/main/LICENSE)

A lightweight, stack-allocated string with fixed capacity and UTF-8 support.  
Ideal for `no_std` environments, embedded systems, and performance-critical code.

The code is formatted according to the [M-RUST](https://github.com/DanilaMint/formating/blob/main/rust.md) standard.

```toml
[dependencies]
microstr = "0.4.1"
```

## What is MicroStr?

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
assert_eq!(s.push_str(" this won't fit entirely"), Err(10)); // Truncated safely
```

You can also use it like a regular `&str` thanks to `Deref`:

```rust
if s.starts_with("Hello") {
    println!("Greeting: {}", s);
}
```

## Cargo Features

Enable optional features in `Cargo.toml`:

```toml
[dependencies]
microstr = { version = "0.4", features = ["std", "serde"] }
```

| Feature | Description |
|--------|-------------|
| `std` (default: on) | Enables `From<String>`, and `Into<String>`. |

## Why MicroStr?

- **Predictable performance**: No allocations, no heap usage.  
- **Memory safety**: Always valid UTF-8, no buffer overflows.  
- **Great for constrained environments**: Embedded, kernels, WASM.  
- **Easy migration**: Drop-in replacement for `String` in many cases.
- **Macro**: Creation via the convenient macro `microstr!`

## Comparison with `heapless::String`

| Feature             | `microstr`               | `heapless::String`       |
|---------------------|--------------------------|--------------------------|
| UTF-8 safety        | ✅ Always valid           | ✅ Always valid           |
| `no_std`            | ✅ Yes                    | ✅ Yes                    |
| Truncation on write | ✅ Yes (safe)             | ❌ Returns `Err` on overflow |
| `const fn` support  | ✅ `from_const`, `new`    | Limited                  |
| Macro convenience   | ✅ `microstr!`            | ❌ No built-in macro     |

`microstr` prioritizes **zero-cost truncation** and **ease of use** in embedded contexts.

## API Documentation

📚 Full documentation: [https://docs.rs/microstr](https://docs.rs/microstr)

## License

[MIT License](./LICENSE)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
