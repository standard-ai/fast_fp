# Fast Floating-Point Math

`fast_fp` provides a set of primitive types that support [fast-math] compiler
optimizations for many operations. These optimizations allow the compiler to
potentially generate faster code by relaxing some of the requirements of [IEEE
754] floating-point arithmetic.

## Examples

```rust
use fast_fp::{FF32, ff32};

// Construct instances of the fast type from std's type using the convenience
// wrapper `ff32` (or `ff64`)
let four = ff32(4.0);

// or using the `From`/`Into` trait
let five: FF32 = 5.0.into();

assert_eq!(four + five, ff32(9.0));

// Most ops are also implemented to work with std's floats too (including PartialEq).
// This makes working with literals easier.
assert_eq!(five + 6.0, 11.0);
assert_eq!(five * 2.0, 10.0);

// Functions can be made generic to accept std or fast types using `num-traits`
use num_traits::real::Real;
fn square<T: Real>(num: T) -> T {
    num * num
}
assert_eq!(square(3.0_f32), 9.0);
assert_eq!(square(five), 25.0);

// If the nalgebra feature (with version suffix) is enabled, interop with
// nalgebra is supported
# #[cfg(feature = "nalgebra_v029")]
use nalgebra_v029 as na;
# #[cfg(feature = "nalgebra_v029")]
assert_eq!(na::Matrix3::repeat(four).sum(), 36.0);
```

# Caveats

### Precision
The fast-math optimizations may result in different outputs than operations
on the standard float primitives like `f32`, particularly where fine-grained
precision is important. Fast-math may allow reordering operations in such a
way that some precision is lost in the overall computation. Note that there are
also cases where fast-math optimizations can _improve_ precision, such as
contracting separate multiplication and addition into a fused multiply-add
operation.

### Performance
Use of this crate's primitives may not be faster than the standard primitives
in all cases. That may be because the generated code is slower in practice, or
because of certain measures taken by this crate to prevent Undefined Behavior
(in particular for comparison heavy code). Users should carefully measure and
benchmark their code to understand whether they actually benefit from use of
these types.

### Finite Math
Many operations have the `finite-math-only` optimization flag enabled. With
this flag, the user must ensure that operations on the fast types **do not**
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
