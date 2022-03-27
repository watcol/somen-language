# somen-language
![status](https://img.shields.io/badge/status-Active-brightgreen?style=flat-square)
[![crates.io](https://img.shields.io/crates/v/somen-language?style=flat-square)](https://crates.io/crates/somen-language)
[![Downloads](https://img.shields.io/crates/d/somen-language?style=flat-square)](https://crates.io/crates/somen-language)
[![Downloads (latest)](https://img.shields.io/crates/dv/somen-language?style=flat-square)](https://crates.io/crates/somen-language)
[![License](https://img.shields.io/crates/l/somen-language?style=flat-square)](https://github.com/watcol/somen-language/blob/main/LICENSE)
![Lint](https://img.shields.io/github/workflow/status/watcol/somen-language/Lint?label=lint&style=flat-square)
![Test](https://img.shields.io/github/workflow/status/watcol/somen-language/Test?label=test&style=flat-square)

Utilities of the somen parser combinator for languages.

## Usage
Add to your `Cargo.toml`:
```toml
[dependencies]
somen-language = "0.1.0"
```

If you are in the `no_std` environment:
```toml
[dependencies.somen-language]
version = "0.1.0"
default-features = false
features = ["alloc"]   # If you have an allocator implementation
```

## Documentation
API Documentations are available on [here](https://docs.rs/somen-language).

## License
This program is licensed under the MIT license.
See [LICENSE](https://github.com/watcol/somen-language/blob/main/LICENSE) for details.
