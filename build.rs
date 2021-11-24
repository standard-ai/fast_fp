fn main() {
    let mut builder = cc::Build::new();

    if !builder.get_compiler().is_like_clang() {
        // if the default/configured cc is not clang, try to call clang manually
        builder.compiler("clang");
    }

    builder.flag("-flto=thin");

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
    builder.flag("-O3");

    #[cfg(feature = "finite-math-only")]
    builder.flag("-ffinite-math-only");

    #[cfg(feature = "associative-math")]
    builder.flag("-fassociative-math");

    #[cfg(feature = "reciprocal-math")]
    builder.flag("-freciprocal-math");

    #[cfg(feature = "no-signed-zeros")]
    builder.flag("-fno-signed-zeros");

    #[cfg(feature = "no-trapping-math")]
    builder.flag("-fno-trapping-math");

    #[cfg(feature = "fp-contract-fast")]
    builder.flag("-ffp-contract=fast");

    // TODO figure out if this works
    //#[cfg(feature = "approx-func")]
    //builder.flag("-Xclang -fapprox-func");

    #[cfg(feature = "denormal-fp-math-preserve-sign")]
    builder.flag("-fdenormal-fp-math=preserve-sign");

    builder.file("src/math/math.c").compile("math")
}
