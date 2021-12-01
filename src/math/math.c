#include <stdbool.h>
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
                                                 \
  __attribute__((always_inline))                 \
  bool eq_ ## RUST_TYPE(C_TYPE a, C_TYPE b) {    \
    return a == b;                               \
  }                                              \
                                                 \
  __attribute__((always_inline))                 \
  bool lt_ ## RUST_TYPE(C_TYPE a, C_TYPE b) {    \
    return a < b;                                \
  }                                              \
                                                 \
  __attribute__((always_inline))                 \
  bool le_ ## RUST_TYPE(C_TYPE a, C_TYPE b) {    \
    return a <= b;                               \
  }                                              \
                                                 \
  __attribute__((always_inline))                 \
  bool gt_ ## RUST_TYPE(C_TYPE a, C_TYPE b) {    \
    return a > b;                                \
  }                                              \
                                                 \
  __attribute__((always_inline))                 \
  bool ge_ ## RUST_TYPE(C_TYPE a, C_TYPE b) {    \
    return a >= b;                               \
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

// FIXME sqrt is not poison safe on some targets
IMPL_UNARY_FUNCTION(float, f32, sqrt, sqrtf)
IMPL_UNARY_FUNCTION(double, f64, sqrt, sqrt)

// FIXME mod is not poison safe, though LLVM frem is
IMPL_BINARY_FUNCTION(float, f32, rem, fmodf)
IMPL_BINARY_FUNCTION(double, f64, rem, fmod)

IMPL_BINARY_FUNCTION(float, f32, max, fmaxf)
IMPL_BINARY_FUNCTION(double, f64, max, fmax)

IMPL_BINARY_FUNCTION(float, f32, min, fminf)
IMPL_BINARY_FUNCTION(double, f64, min, fmin)
