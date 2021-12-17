fn main() {
    let mut builder = cc::Build::new();

    if !builder.get_compiler().is_like_clang() {
        // if the default/configured cc is not clang, try to call clang manually
        builder.compiler("clang");
    }

    builder.warnings_into_errors(true);
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
    builder.opt_level(3);

    #[cfg(not(feature = "no-associative-math"))]
    builder.flag("-fassociative-math");

    #[cfg(not(feature = "no-reciprocal-math"))]
    builder.flag("-freciprocal-math");

    #[cfg(not(feature = "signed-zeros"))]
    builder.flag("-fno-signed-zeros");

    #[cfg(not(feature = "trapping-math"))]
    builder.flag("-fno-trapping-math");

    #[cfg(not(feature = "fp-contract-on"))]
    builder.flag("-ffp-contract=fast");

    // -fapprox-func isn't currently available in the driver, but it is in clang itself
    // https://reviews.llvm.org/D106191
    #[cfg(not(feature = "no-approx-func"))]
    builder.flag("-Xclang").flag("-fapprox-func");

    #[cfg(not(feature = "math-errno"))]
    builder.flag("-fno-math-errno");

    // poison_unsafe must be compiled without finite-math-only
    // see its docs for details
    poison_unsafe(builder.clone());

    #[cfg(not(feature = "no-finite-math-only"))]
    builder.flag("-ffinite-math-only");

    poison_safe(builder);
}

fn poison_unsafe(mut builder: cc::Build) {
    builder
        .file("src/math/poison_unsafe.c")
        .compile("poison_unsafe")
}

fn poison_safe(mut builder: cc::Build) {
    builder
        .file("src/math/poison_safe.c")
        .compile("poison_safe")
}
