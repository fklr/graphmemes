# graphmemes

A `#![no_std]` compatible, zero-allocation Unicode grapheme cluster iterator.

## Key Features

- **Zero-Allocation Design**: Fixed-size buffer implementation with no heap allocations
- **Unicode Processing**:
  - Complete UAX #29 grapheme cluster boundary detection
  - Efficient bit pattern-based boundary rules
  - Support for combining marks, emoji, and ZWJ sequences
  - Regional indicator (flag) handling
  - RTL text with combining marks
- **ANSI Support**:
  - Optional ANSI escape sequence counting
  - Safe sequence validation and processing

## #![no_std] Support

This crate is `#![no_std]` compatible and makes zero heap allocations. All operations use fixed-size buffers and stack-only data structures.

## Dependencies

```toml
[dependencies]
owo-colors = "4.1.0"
```

## License

This project is licensed under [![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](LICENSE)

