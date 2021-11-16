fn main() {
    let mut builder = cc::Build::new();

    if !builder.get_compiler().is_like_clang() {
        // if the default/configured cc is not clang, try to call clang manually
        builder.compiler("clang");
    }

    builder.flag("-O3").flag("-flto=thin");

    build_ll(builder.clone());
    build_c(builder);
}

fn build_ll(mut builder: cc::Build) {
    // the ll files are written bare, let the compiler override module annotations and don't warn
    // about it
    builder.flag("-Wno-override-module");

    builder.file("src/poison/freeze.ll").compile("freeze");
}

fn build_c(mut builder: cc::Build) {
    builder
        .file("src/math/math.c")
        .flag("-ffinite-math-only")
        .flag("-fassociative-math")
        .flag("-freciprocal-math")
        .flag("-fno-signed-zeros")
        .flag("-fno-trapping-math")
        .flag("-ffp-contract=fast")
        .compile("math")
}
