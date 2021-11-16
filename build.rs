fn main() {
    let mut builder = cc::Build::new();

    if !builder.get_compiler().is_like_clang() {
        // if the default/configured cc is not clang, try to call clang manually
        builder.compiler("clang");
    }

    builder
        .file("src/math/math.c")
        .flag("-O3")
        .flag("-flto=thin")
        .flag("-ffinite-math-only")
        .flag("-fassociative-math")
        .flag("-freciprocal-math")
        .flag("-fno-signed-zeros")
        .flag("-fno-trapping-math")
        .flag("-ffp-contract=fast")
        .compile("math")
}
