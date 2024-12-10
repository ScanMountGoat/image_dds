fn main() {
    // Renamed to bcdec.c since cc doesn't like header only.
    // Don't enable defines marking functions "static" to avoid linking issues.
    cc::Build::new()
        .file("src/bcdec.c")
        .define("BCDEC_IMPLEMENTATION", None)
        .define("BCDEC_BC4BC5_PRECISE", None)
        .compile("bcdec");
}
