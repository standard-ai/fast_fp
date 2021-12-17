/*
 * The functions in this file are ones which can safely accept poison values in
 * their input arguments without triggering any UB[1]. Because they can accept
 * poison values, any fast-math optimizations are valid for this file, and rust
 * code can still safely call it without precautions like freezing.
 *
 * [1]: https://llvm.org/docs/LangRef.html#poison-values
 */

#include <math.h>

#define IMPL_OPERATIONS(C_TYPE, RUST_TYPE)       \
  __attribute__((always_inline))                 \
  C_TYPE add_ ## RUST_TYPE(C_TYPE a, C_TYPE b) { \
    return a + b;                                \
  }                                              \
                                                 \
  __attribute__((always_inline))                 \
  C_TYPE sub_ ## RUST_TYPE(C_TYPE a, C_TYPE b) { \
    return a - b;                                \
  }                                              \
                                                 \
  __attribute__((always_inline))                 \
  C_TYPE mul_ ## RUST_TYPE(C_TYPE a, C_TYPE b) { \
    return a * b;                                \
  }                                              \
                                                 \
  __attribute__((always_inline))                 \
  C_TYPE div_ ## RUST_TYPE(C_TYPE a, C_TYPE b) { \
    return a / b;                                \
  }                                              \
                                                 \
  __attribute__((always_inline))                 \
  C_TYPE neg_ ## RUST_TYPE(C_TYPE a) {           \
    return -a;                                   \
  }                                              \

#define IMPL_UNARY_FUNCTION(C_TYPE, RUST_TYPE, FN_NAME, FN_IMPL) \
  __attribute__((always_inline))                                 \
  C_TYPE FN_NAME ## _ ## RUST_TYPE(C_TYPE a) {                   \
    return FN_IMPL(a);                                           \
  }                                                              \

#define IMPL_BINARY_FUNCTION(C_TYPE, RUST_TYPE, FN_NAME, FN_IMPL) \
  __attribute__((always_inline))                                  \
  C_TYPE FN_NAME ## _ ## RUST_TYPE(C_TYPE a, C_TYPE b) {          \
    return FN_IMPL(a, b);                                         \
  }                                                               \

IMPL_OPERATIONS(float, f32)
IMPL_OPERATIONS(double, f64)

IMPL_UNARY_FUNCTION(float, f32, abs, fabsf)
IMPL_UNARY_FUNCTION(double, f64, abs, fabs)

IMPL_BINARY_FUNCTION(float, f32, copysign, copysignf)
IMPL_BINARY_FUNCTION(double, f64, copysign, copysign)

IMPL_BINARY_FUNCTION(float, f32, max, fmaxf)
IMPL_BINARY_FUNCTION(double, f64, max, fmax)

IMPL_BINARY_FUNCTION(float, f32, min, fminf)
IMPL_BINARY_FUNCTION(double, f64, min, fmin)

__attribute__((always_inline))
float powi_f32(float a, int b) {
  return __builtin_powif(a, b);
}

__attribute__((always_inline))
double powi_f64(double a, int b) {
  return __builtin_powi(a, b);
}

__attribute__((always_inline))
float clamp_f32(float a, float min, float max) {
  // under -O3 these comparisons are compiled to selects which, unlike
  // branches, propagate poison without UB
  if(a < min) {
    a = min;
  }
  if(a > max) {
    a = max;
  }
  return a;
}

__attribute__((always_inline))
double clamp_f64(double a, double min, double max) {
  if(a < min) {
    a = min;
  }
  if(a > max) {
    a = max;
  }
  return a;
}

