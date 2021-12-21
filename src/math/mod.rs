use crate::{FF32, FF64};
use core::ops::{Add, Div, Mul, Neg, Rem, Sub};
use paste::paste;

macro_rules! impl_binary_refs {
    ($lhs:ident, $rhs:ident, $op_trait:ident, $op_fn:ident) => {
        impl $op_trait<$rhs> for &$lhs {
            type Output = <$lhs as $op_trait<$rhs>>::Output;

            #[inline]
            fn $op_fn(self, other: $rhs) -> Self::Output {
                (*self).$op_fn(other)
            }
        }
        impl $op_trait<&$rhs> for $lhs {
            type Output = <$lhs as $op_trait<$rhs>>::Output;

            #[inline]
            fn $op_fn(self, other: &$rhs) -> Self::Output {
                self.$op_fn(*other)
            }
        }
        impl $op_trait<&$rhs> for &$lhs {
            type Output = <$lhs as $op_trait<$rhs>>::Output;

            #[inline]
            fn $op_fn(self, other: &$rhs) -> Self::Output {
                (*self).$op_fn(*other)
            }
        }
    };
}

macro_rules! impl_fast_ops {
    ($fast_ty:ident, $base_ty: ident: $($op_trait:ident, $op_fn:ident, $op_impl:ident,)*) => {
        $(
            impl $op_trait <$fast_ty> for $fast_ty {
                type Output = $fast_ty;

                #[inline(always)]
                fn $op_fn(self, other: $fast_ty) -> Self::Output {
                    unsafe { $op_impl(self, other) }
                }
            }

            impl $op_trait <$base_ty> for $fast_ty {
                type Output = $fast_ty;

                #[inline(always)]
                fn $op_fn(self, other: $base_ty) -> Self::Output {
                    self.$op_fn(<$fast_ty>::new(other))
                }
            }

            impl $op_trait <$fast_ty> for $base_ty {
                type Output = $fast_ty;

                #[inline(always)]
                fn $op_fn(self, other: $fast_ty) -> Self::Output {
                    <$fast_ty>::new(self).$op_fn(other)
                }
            }

            impl_binary_refs! { $fast_ty, $fast_ty, $op_trait, $op_fn }
            impl_binary_refs! { $fast_ty, $base_ty, $op_trait, $op_fn }
            impl_binary_refs! { $base_ty, $fast_ty, $op_trait, $op_fn }
        )*
    };
}

macro_rules! poison_safe_fns {
    ($fast_ty:ident, $base_ty:ident:
     $(fn $fn:ident(self $(, $arg:ident : Self)*) -> Self;)*) => {
        paste! {
            $(
                #[link(name = "poison_safe")]
                extern "C" {
                    // functions in the poison_safe lib can accept poison args.
                    // because the fast types are (transitively) repr(transparent) over the
                    // primitive type, we can pass them directly over FFI
                    fn [<$fn _ $base_ty>](a: $fast_ty $(, $arg: $fast_ty)*) -> $fast_ty;
                }
            )*

            impl $fast_ty {
                $(
                    #[inline]
                    pub fn $fn(self $(, $arg: Self)*) -> Self {
                        unsafe { [<$fn _ $base_ty>](self $(, $arg)*) }
                    }
                )*
            }
        }
    }
}

macro_rules! poison_unsafe_fns {
    ($fast_ty:ident, $base_ty:ident:
     $(fn $fn:ident(self $(, $arg:ident : Self)*) -> Self;)*) => {
        paste! {
            $(
                #[link(name = "poison_unsafe")]
                extern "C" {
                    // functions in the poison_unsafe lib must have their arguments frozen, which
                    // is best expressed as accepting the base type instead of the fast type
                    fn [<$fn _ $base_ty>](a: $base_ty $(, $arg: $base_ty)*) -> $fast_ty;
                }

            )*

            impl $fast_ty {
                $(
                    #[inline]
                    pub fn $fn(self $(, $arg: Self)*) -> Self {
                        unsafe { [<$fn _ $base_ty>](self.freeze_raw() $(, $arg.freeze_raw())*) }
                    }
                )*
            }
        }
    }
}

macro_rules! impl_extern_math {
    ($fast_ty:ident, $base_ty:ident) => {
        poison_safe_fns! {
            $fast_ty, $base_ty:
            fn abs(self) -> Self;
            fn copysign(self, other: Self) -> Self;
            fn max(self, other: Self) -> Self;
            fn min(self, other: Self) -> Self;
        }

        poison_unsafe_fns! {
            $fast_ty, $base_ty:
            fn acos(self) -> Self;
            fn acosh(self) -> Self;
            fn asin(self) -> Self;
            fn asinh(self) -> Self;
            fn atan(self) -> Self;
            fn atan2(self, other: Self) -> Self;
            fn atanh(self) -> Self;
            fn cbrt(self) -> Self;
            fn ceil(self) -> Self;
            fn cos(self) -> Self;
            fn cosh(self) -> Self;
            fn exp(self) -> Self;
            fn exp2(self) -> Self;
            fn floor(self) -> Self;
            fn exp_m1(self) -> Self;
            fn ln(self) -> Self;
            fn ln_1p(self) -> Self;
            fn log2(self) -> Self;
            fn log10(self) -> Self;
            fn powf(self, n: Self) -> Self;
            fn round(self) -> Self;
            fn sin(self) -> Self;
            fn sinh(self) -> Self;
            fn sqrt(self) -> Self;
            fn tan(self) -> Self;
            fn tanh(self) -> Self;
            fn trunc(self) -> Self;
        }

        paste! {
            #[link(name = "poison_safe")]
            extern "C" {
                fn [<add_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
                fn [<sub_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
                fn [<mul_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
                fn [<div_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
                fn [<neg_ $base_ty>](a: $fast_ty) -> $fast_ty;

                fn [<clamp_ $base_ty>](a: $fast_ty, min: $fast_ty, max: $fast_ty) -> $fast_ty;
                fn [<powi_ $base_ty>](a: $fast_ty, b: i32) -> $fast_ty;
            }

            #[link(name = "poison_unsafe")]
            extern "C" {
                fn [<rem_ $base_ty>](a: $base_ty, b: $base_ty) -> $fast_ty;
            }

            // a few functions are special cases and aren't defined in submacros
            impl $fast_ty {
                #[inline]
                pub fn clamp(self, min: Self, max: Self) -> Self {
                    assert!(min <= max);
                    unsafe { [<clamp_ $base_ty>](self, min, max) }
                }

                #[inline]
                pub fn powi(self, n: i32) -> Self {
                    unsafe { [<powi_ $base_ty>](self, n) }
                }
            }

            impl_fast_ops! {
                $fast_ty, $base_ty:
                Add, add, [<add_ $base_ty>],
                Sub, sub, [<sub_ $base_ty>],
                Mul, mul, [<mul_ $base_ty>],
                Div, div, [<div_ $base_ty>],
            }

            impl Neg for $fast_ty {
                type Output = Self;

                #[inline(always)]
                fn neg(self) -> Self::Output {
                    unsafe { [<neg_ $base_ty>](self) }
                }
            }

            impl Neg for &$fast_ty {
                type Output = <$fast_ty as Neg>::Output;

                #[inline]
                fn neg(self) -> Self::Output {
                    -(*self)
                }
            }

            impl Rem <$fast_ty> for $fast_ty {
                type Output = $fast_ty;

                #[inline(always)]
                fn rem(self, other: $fast_ty) -> Self::Output {
                    unsafe { [<rem_ $base_ty>](self.freeze_raw(), other.freeze_raw()) }
                }
            }

            impl Rem <$base_ty> for $fast_ty {
                type Output = $fast_ty;

                #[inline(always)]
                fn rem(self, other: $base_ty) -> Self::Output {
                    unsafe { [<rem_ $base_ty>](self.freeze_raw(), other) }
                }
            }

            impl Rem <$fast_ty> for $base_ty {
                type Output = $fast_ty;

                #[inline(always)]
                fn rem(self, other: $fast_ty) -> Self::Output {
                    unsafe { [<rem_ $base_ty>](self, other.freeze_raw()) }
                }
            }

            impl_binary_refs! { $fast_ty, $fast_ty, Rem, rem }
            impl_binary_refs! { $fast_ty, $base_ty, Rem, rem }
            impl_binary_refs! { $base_ty, $fast_ty, Rem, rem }
        }
    };
}

impl_extern_math! { FF32, f32 }
impl_extern_math! { FF64, f64 }
