# Fast Floating-Point Math

`fast_fp` provides a set of primitive types that support [fast-math] compiler
optimizations for many operations. These optimizations allow the compiler to
potentially generate faster code by relaxing some of the requirements of [IEEE
754] floating-point arithmetic.

This may result in different outputs than operations on the standard float
primitives like `f32`, particularly where fine-grained precision is important.
`fast-math` may allow reordering operations in such a way that some precision
is lost in the overall computation. Note that there are also cases where
fast-math optimizations can _improve_ precision, such as contracting separate
multiplication and addition into a fused multiply-add operation.

## Caveats

### Performance
Use of this crate's primitives may not be faster than the standard primitives
in all cases. That may be because the generated code is slower in practice, or
because of certain measures taken by this crate to prevent Undefined Behavior
(in particular for comparison heavy code). Users should carefully measure and
benchmark their code to understand whether they actually benefit from use of
these types.

### Finite Math
By default, the `finite-math-only` optimization flag is enabled. With this
enabled, the user must ensure that operations on the fast types **do not**
involve infinite or NaN values. If the arguments to an operation are, or the
results of an operation _would_ be, `+inf`, `-inf`, or `NaN`, then the
operation's result value is unspecified. This crate goes to lengths to ensure
that such an operation is not UB in the strict sense, but the output is free to
be any representable value of the output type, and may not be a fixed value at
all.

### Building
`fast_fp` enables fast-math optimizations by calling C code which was compiled
with these optimizations enabled; additionally, some LLVM IR is used to prevent
triggering UB that is otherwise possible with these optimizations. As a
consequence, building this crate requires `clang` to be installed _and_
requires the final binary to be linked using cross-language LTO to achieve the
performance benefits.

This LTO requires a version of clang compatible with the LLVM version used by
rustc. To find the necessary LLVM version, check rustc's version info in
verbose mode:

```shell
$ rustc -vV
rustc 1.56.0 (09c42c458 2021-10-18)
binary: rustc
commit-hash: 09c42c45858d5f3aedfa670698275303a3d19afa
commit-date: 2021-10-18
host: x86_64-unknown-linux-gnu
release: 1.56.0
LLVM version: 13.0.0 # <--- see the version here
```

Then build and link using a `clang` and `lld` with the corresponding version:

```shell
$ CC="clang-13" \
RUSTFLAGS="-Clinker-plugin-lto -Clinker=clang-13 -Clink-arg=-fuse-ld=lld-13" \
cargo build
```

For simplicity, these arguments can be stored in a [cargo config] file

```toml
[env]
CC = "clang-13"

[build]
rustflags = ["-Clinker-plugin-lto", "-Clinker=clang-13", "-Clink-arg=-fuse-ld=lld-13"]
```

Although rustc does not always use an official LLVM release version, it's
typically close enough to be interoperable with the official clang and LLVM
releases of the same version number.

[fast-math]: https://llvm.org/docs/LangRef.html#fast-math-flags
[IEEE 754]: https://en.wikipedia.org/wiki/IEEE_754
[cargo config]: https://doc.rust-lang.org/cargo/reference/config.html
