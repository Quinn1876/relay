/// REFERENCE BLOG for building an ffi crate: https://medium.com/dwelo-r-d/using-c-libraries-in-rust-13961948c72a

fn main() {
    // pkg_config::Config::new()
    //     .atleast_version("1.2")
    //     .probe
    let src = [
        "../canbus-canota/canota.c",
        "../canbus-canota/crc32.c"
    ];
    let mut builder = cc::Build::new();
    let build = builder
        .files(src.iter())
        .include("../canbus-canota");

    build.compile("canota");
}
