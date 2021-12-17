/*
 * The functions in this file are ones which *cannot* safely accept poison
 * values in their input arguments without potentially triggering UB[1]. To
 * make them safe to call from rust, two steps must be taken:
 *
 * 1. Arguments passed to these functions must be frozen before the call, so
 * that poison values do not enter the function.
 *
 * 2. fast-math flags which potentially produce poison values must be disabled
 * when compiling this file, so that poison values cannot be generated within
 * them. Currently, this applies only to `finite-math-only`: NaN and +/-inf
 * must be honored, because the `finite-math-only` flag is the only fast-math
 * flag that can produce LLVM poison as of this writing.
 *
 * These two constraints potentially prevent some optimizations. However this
 * seems like the best compromise between safety and performance, to allow an
 * ergonomic rust interface without labelling many methods `unsafe` and
 * requiring the users to police their input values.
 *
 * [1]: https://llvm.org/docs/LangRef.html#poison-values
 */

#include <math.h>

#define IMPL_UNARY_FUNCTION(C_TYPE, RUST_TYPE, FN_NAME, FN_IMPL) \
  __attribute__((always_inline))                                 \
  C_TYPE FN_NAME ## _ ## RUST_TYPE(C_TYPE a) {                   \
    return FN_IMPL(a);                                           \
  }                                                              \

#define IMPL_UNARY(DOUBLE_FN)                                \
  IMPL_UNARY_FUNCTION(double, f64, DOUBLE_FN, DOUBLE_FN)     \
  IMPL_UNARY_FUNCTION(float, f32, DOUBLE_FN, DOUBLE_FN ## f) \

#define IMPL_BINARY_FUNCTION(C_TYPE, RUST_TYPE, FN_NAME, FN_IMPL) \
  __attribute__((always_inline))                                  \
  C_TYPE FN_NAME ## _ ## RUST_TYPE(C_TYPE a, C_TYPE b) {          \
    return FN_IMPL(a, b);                                         \
  }                                                               \

#define IMPL_BINARY(DOUBLE_FN)                                \
  IMPL_BINARY_FUNCTION(double, f64, DOUBLE_FN, DOUBLE_FN)     \
  IMPL_BINARY_FUNCTION(float, f32, DOUBLE_FN, DOUBLE_FN ## f) \

IMPL_UNARY(acos)
IMPL_UNARY(acosh)
IMPL_UNARY(asin)
IMPL_UNARY(asinh)
IMPL_UNARY(atan)
IMPL_BINARY(atan2)
IMPL_UNARY(atanh)
IMPL_UNARY(cbrt)
IMPL_UNARY(ceil)
IMPL_UNARY(cos)
IMPL_UNARY(cosh)
IMPL_UNARY(exp)
IMPL_UNARY(exp2)
IMPL_UNARY(floor)

IMPL_UNARY_FUNCTION(double, f64, exp_m1, expm1)
IMPL_UNARY_FUNCTION(float, f32, exp_m1, expm1f)

IMPL_BINARY_FUNCTION(double, f64, rem, fmod)
IMPL_BINARY_FUNCTION(float, f32, rem, fmodf)

IMPL_UNARY_FUNCTION(double, f64, ln, log)
IMPL_UNARY_FUNCTION(float, f32, ln, logf)

IMPL_UNARY_FUNCTION(double, f64, ln_1p, log1p)
IMPL_UNARY_FUNCTION(float, f32, ln_1p, log1pf)

IMPL_UNARY(log2)
IMPL_UNARY(log10)

IMPL_BINARY_FUNCTION(double, f64, powf, pow)
IMPL_BINARY_FUNCTION(float, f32, powf, powf)

IMPL_UNARY(round)
IMPL_UNARY(sin)
IMPL_UNARY(sinh)
IMPL_UNARY(sqrt)
IMPL_UNARY(tan)
IMPL_UNARY(tanh)
IMPL_UNARY(trunc)

