# Fast Floating-Point Math

`fast_fp` provides a set of primitive types that support [fast-math]
optimizations for many operations. These optimizations allow the compiler to
potentially generate faster code by relaxing some of the requirements of [IEEE
754] floating-point arithmetic.

This may result in different outputs than operations on the standard float
primitives like `f32`, particularly where fine-grained precision is important.
`fast-math` may allow reordering operations in such a way that some precision
is lost in the overall computation. Note that there are also cases where
fast-math optimizations can _improve_ precision, such as contracting separate
multiplication and addition into a fused multiply-add operation.

## Limitations

In order to enable these optimizations safely, certain requirements must be
observed:

- Operations **MUST NOT** involve infinite or NaN values. If the arguments to an
	operation are, or the results of an operation _would_ be, `+inf`, `-inf`,
	or `NaN`, then the operation's result value is unspecified. This crate goes
	to lengths to ensure that such an operation is not Undefined Behavior in the
	strict sense, but the output is free to be any representable value of the
	output type, and may not be a fixed value at all.
- Use of this crate's primitives may not be faster than the standard primitives
	in all cases. That may be because the generated code is slower in practice,
	or because of certain measures taken by this crate to prevent UB (in
	particular for comparison heavy code). Users should carefully measure and
	benchmark their code to understand whether they actually benefit from use of
	these types.
- The safety of this crate is only assessed against rustc's LLVM code
	generation. This crate should not be used with alternative code generators
	such as cranelift or GCC
- Signed-ness of zeros may be treated as insignificant and not preserved

[TODO]: # (is there a way to detect the code generator at build time?)

[fast-math]: https://llvm.org/docs/LangRef.html#fast-math-flags
[IEEE 754]: https://en.wikipedia.org/wiki/IEEE_754
