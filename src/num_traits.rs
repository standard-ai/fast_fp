#![cfg(feature = "num-traits")]
#![cfg_attr(docsrs, doc(cfg(feature = "num-traits")))]
use crate::{FF32, FF64};
use core::num::FpCategory;

macro_rules! forward_freeze_ty {
    ($fast_ty:ident, $base_ty:ident
     $(
         $(#[$attr:meta])*
         $vis:vis fn $fn_name:ident (&self) -> $ret_ty:ty ;
     )*) => {
        $(
            $(#[$attr])*
            #[inline]
            $vis fn $fn_name(&self) -> $ret_ty {
                <$base_ty>::$fn_name(&self.freeze_raw())
            }
        )*
    }
}

macro_rules! forward_self {
    ($fast_ty:ident, $base_ty:ident
     $(
         $(#[$attr:meta])*
         $vis:vis fn $fn_name:ident (self $(, $arg:ident : $arg_ty:ty)* ) -> $ret_ty:ty ;
     )*) => {
        $(
            $(#[$attr])*
            #[inline]
            $vis fn $fn_name(self $(, $arg : $arg_ty)*) -> $ret_ty {
                <$fast_ty>::$fn_name(self $(, $arg)* )
            }
        )*
     };

    ($fast_ty:ident, $base_ty:ident
     $(
         $(#[$attr:meta])*
         $vis:vis fn $fn_name:ident (&self $(, $arg:ident : $arg_ty:ty)* ) -> $ret_ty:ty ;
     )*) => {
        $(
            $(#[$attr])*
            #[inline]
            $vis fn $fn_name(&self $(, $arg : $arg_ty)*) -> $ret_ty {
                <$fast_ty>::$fn_name(*self $(, $arg)* )
            }
        )*
     };
}

pub(crate) use forward_self;

macro_rules! opt_from_base {
    ($fast_ty:ident, $base_ty:ident
     $(
         $(#[$attr:meta])*
         $vis:vis fn $fn_name:ident ($($arg:ident : $arg_ty:ty),*) -> Option<Self> ;
     )*) => {
        $(
            $(#[$attr])*
            #[inline]
            $vis fn $fn_name($($arg : $arg_ty),*) -> Option<Self> {
                Some(<$fast_ty>::new(<$base_ty>::$fn_name($($arg),*)?))
            }
        )*
    }
}

macro_rules! impl_num_traits {
    ($fast_ty:ident, $base_ty:ident) => {
        impl num_traits::One for $fast_ty {
            #[inline(always)]
            fn one() -> Self {
                Self::ONE
            }

            #[inline]
            fn is_one(&self) -> bool {
                self.freeze_raw() == 1.0
            }
        }

        impl num_traits::Zero for $fast_ty {
            #[inline(always)]
            fn zero() -> Self {
                Self::ZERO
            }

            #[inline]
            fn is_zero(&self) -> bool {
                self.freeze_raw() == 0.0
            }
        }

        impl num_traits::Bounded for $fast_ty {
            #[inline]
            fn max_value() -> Self {
                Self::MAX
            }
            #[inline]
            fn min_value() -> Self {
                Self::MIN
            }
        }

        impl num_traits::Signed for $fast_ty {
            forward_self! {
                $fast_ty, $base_ty
                fn abs(&self) -> Self ;
                fn signum(&self) -> Self ;
            }

            forward_freeze_self! {
                $fast_ty, $base_ty
                #[allow(deprecated)]
                fn abs_sub(&self, other: &Self) -> Self;
            }

            #[inline]
            fn is_positive(&self) -> bool {
                self.freeze_raw() > 0.0
            }

            #[inline]
            fn is_negative(&self) -> bool {
                self.freeze_raw() < 0.0
            }
        }

        impl num_traits::Num for $fast_ty {
            type FromStrRadixErr = <$base_ty as num_traits::Num>::FromStrRadixErr;

            fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                Ok(<$fast_ty>::new(
                    <$base_ty as num_traits::Num>::from_str_radix(str, radix)?,
                ))
            }
        }

        impl num_traits::ToPrimitive for $fast_ty {
            forward_freeze_ty! {
                $fast_ty, $base_ty
                fn to_isize(&self) -> Option<isize> ;
                fn to_i8(&self) -> Option<i8> ;
                fn to_i16(&self) -> Option<i16> ;
                fn to_i32(&self) -> Option<i32> ;
                fn to_i64(&self) -> Option<i64> ;
                fn to_i128(&self) -> Option<i128> ;

                fn to_usize(&self) -> Option<usize> ;
                fn to_u8(&self) -> Option<u8> ;
                fn to_u16(&self) -> Option<u16> ;
                fn to_u32(&self) -> Option<u32> ;
                fn to_u64(&self) -> Option<u64> ;
                fn to_u128(&self) -> Option<u128> ;

                fn to_f32(&self) -> Option<f32> ;
                fn to_f64(&self) -> Option<f64> ;
            }
        }

        impl num_traits::FromPrimitive for $fast_ty {
            opt_from_base! {
                $fast_ty, $base_ty
                fn from_isize(n: isize) -> Option<Self> ;
                fn from_i8(n: i8) -> Option<Self> ;
                fn from_i16(n: i16) -> Option<Self> ;
                fn from_i32(n: i32) -> Option<Self> ;
                fn from_i64(n: i64) -> Option<Self> ;
                fn from_i128(n: i128) -> Option<Self> ;

                fn from_usize(n: usize) -> Option<Self> ;
                fn from_u8(n: u8) -> Option<Self> ;
                fn from_u16(n: u16) -> Option<Self> ;
                fn from_u32(n: u32) -> Option<Self> ;
                fn from_u64(n: u64) -> Option<Self> ;
                fn from_u128(n: u128) -> Option<Self> ;

                fn from_f32(n: f32) -> Option<Self> ;
                fn from_f64(n: f64) -> Option<Self> ;
            }
        }

        impl num_traits::NumCast for $fast_ty {
            #[inline]
            fn from<N: num_traits::ToPrimitive>(n: N) -> Option<Self> {
                Some(<$fast_ty>::new(<$base_ty as num_traits::NumCast>::from(n)?))
            }
        }

        /// Because inf and nan are prohibited, the `fast_fp` types correspond more to the `Real`
        /// trait than the `Float` trait. However in practice some libs require a Float bound when
        /// they could really use a Real, which would restrict using the `fast_fp` types.
        impl num_traits::Float for $fast_ty {
            /// Panics because NaN values are not supported
            #[inline]
            fn nan() -> Self {
                panic!(concat!(
                    stringify!($fast_ty),
                    " does not support NaN values"
                ));
            }

            /// Panics because infinite values are not supported
            ///
            /// Consider using [`max_value`](num_traits::Float::max_value) as appropriate instead
            #[inline]
            fn infinity() -> Self {
                panic!(concat!(
                    stringify!($fast_ty),
                    " does not support infinite values. Consider using `max_value` for comparisons"
                ));
            }

            /// Panics because infinite values are not supported
            ///
            /// Consider using [`min_value`](num_traits::Float::min_value) as appropriate instead
            #[inline]
            fn neg_infinity() -> Self {
                panic!(concat!(
                    stringify!($fast_ty),
                    " does not support infinite values. Consider using `min_value` for comparisons"
                ));
            }

            #[inline]
            fn neg_zero() -> Self {
                -Self::ZERO
            }

            #[inline]
            fn min_value() -> Self {
                $fast_ty::MIN
            }

            #[inline]
            fn min_positive_value() -> Self {
                $fast_ty::MIN_POSITIVE
            }

            #[inline]
            fn max_value() -> Self {
                $fast_ty::MAX
            }

            #[inline]
            fn epsilon() -> Self {
                <$fast_ty>::new($base_ty::EPSILON)
            }

            #[inline]
            fn is_nan(self) -> bool {
                false
            }

            #[inline]
            fn is_infinite(self) -> bool {
                false
            }

            #[inline]
            fn is_finite(self) -> bool {
                true
            }

            forward_self! {
                $fast_ty, $base_ty
                fn is_normal(self) -> bool;
                fn classify(self) -> FpCategory;
                fn floor(self) -> Self;
                fn ceil(self) -> Self;
                fn round(self) -> Self;
                fn trunc(self) -> Self;
                fn fract(self) -> Self;
                fn abs(self) -> Self;
                fn signum(self) -> Self;
                fn is_sign_positive(self) -> bool;
                fn is_sign_negative(self) -> bool;
                fn mul_add(self, a: Self, b: Self) -> Self;
                fn recip(self) -> Self;
                fn powi(self, n: i32) -> Self;
                fn powf(self, n: Self) -> Self;
                fn sqrt(self) -> Self;
                fn exp(self) -> Self;
                fn exp2(self) -> Self;
                fn ln(self) -> Self;
                fn log(self, base: Self) -> Self;
                fn log2(self) -> Self;
                fn log10(self) -> Self;
                fn max(self, other: Self) -> Self;
                fn min(self, other: Self) -> Self;
                fn cbrt(self) -> Self;
                fn hypot(self, other: Self) -> Self;
                fn sin(self) -> Self;
                fn cos(self) -> Self;
                fn tan(self) -> Self;
                fn asin(self) -> Self;
                fn acos(self) -> Self;
                fn atan(self) -> Self;
                fn atan2(self, other: Self) -> Self;
                fn sin_cos(self) -> (Self, Self);
                fn exp_m1(self) -> Self;
                fn ln_1p(self) -> Self;
                fn sinh(self) -> Self;
                fn cosh(self) -> Self;
                fn tanh(self) -> Self;
                fn asinh(self) -> Self;
                fn acosh(self) -> Self;
                fn atanh(self) -> Self;
                fn to_degrees(self) -> Self;
                fn to_radians(self) -> Self;
            }

            forward_freeze_self! {
                $fast_ty, $base_ty
                #[allow(deprecated)]
                fn abs_sub(self, other: Self) -> Self;
            }

            #[inline]
            fn integer_decode(self) -> (u64, i16, i8) {
                <$base_ty as num_traits::Float>::integer_decode(self.freeze_raw())
            }
        }
    };
}

impl_num_traits! { FF32, f32 }
impl_num_traits! { FF64, f64 }
