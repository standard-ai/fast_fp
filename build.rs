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

    // TODO control flags with generics
    builder.flag("-fassociative-math");
    builder.flag("-freciprocal-math");
    builder.flag("-fno-signed-zeros");
    builder.flag("-fno-trapping-math");
    builder.flag("-ffp-contract=fast");

    // -fapprox-func isn't currently available in the clang driver (fixed with
    // https://reviews.llvm.org/D106191 but need an updated version), but it is in clang itself.
    // This means it can be toggled using the `-Xclang <arg>` option.
    //
    // The clang version on docs.rs doesn't recognize -fapprox-func even with -Xclang :shrug:
    // We could attempt to check flag support using cc's flag_if_supported, but this is a compound
    // flag which doesn't seem to mix well with the is_supported api checks. Instead, do the dumb
    // thing and don't enable this flag when compiling on docs.rs. That way, normal users should at
    // least get a slightly informative error if they have an incompatible clang
    match std::env::var("DOCS_RS") {
        Err(std::env::VarError::NotPresent) => {
            builder.flag("-Xclang").flag("-fapprox-func");
        }
        Ok(_) => {}
        Err(err) => panic!("{}", err),
    }

    builder.flag("-fno-math-errno");

    // poison_unsafe must be compiled without finite-math-only
    // see its docs for details
    poison_unsafe(builder.clone());

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
