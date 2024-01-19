# bcdec_rs
A pure Rust port of the [bcdec](https://github.com/iOrange/bcdec) C library using only safe code.  
BC1, BC2, BC3, BC4, BC5, BC6H, and BC7 are supported.

The Rust implementation is fuzzed against bindings to the original C code for arbitrary input blocks to test for identical behavior.