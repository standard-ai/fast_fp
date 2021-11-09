#![doc = include_str!("../README.md")]
#![feature(core_intrinsics)] // intrinsics for the fast math
#![feature(asm)] // asm used to emulate freeze
use core::{
    cmp, fmt,
    intrinsics::{fadd_fast, fdiv_fast, fmul_fast, frem_fast, fsub_fast},
    iter::{Product, Sum},
    num::FpCategory,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

mod poison;
use poison::MaybePoison;

/// The error returned by the checked constructors of [`FF32`] and [`FF64`]
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidValueError {
    _priv: (),
}

impl fmt::Display for InvalidValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("value may not be infinite or NaN")
    }
}

impl std::error::Error for InvalidValueError {}

// The big challenge with fast-math in general is avoiding UB, and to a lesser extent unspecified
// values. LLVM's fast operations document "poison" behavior when given invalid inputs; poison
// values have a relatively consistent behavior (stuff like transitivity), defined cases for UB,
// and importantly can be limited in scope by freezing to a fixed value.
//
// This library handles poison by limiting its reach to only the pure arithmetic operations on the
// wrapper float types. Any arbitrary FF32 is considered possibly invalid (containing +-inf or NaN)
// because it's not feasible to track validity (without running all operations in parallel with
// unfast-math and thus negating any possible improvement). Float add/sub/mul/div/rem are permitted
// on the possibly poison values (as documented by LLVM), producing transitively poison results,
// then wrapped in FF32. Any other operations require the value to be not-poison in order to be
// not-UB: anything like comparison/printing/conversion/casting/etc is done on frozen copies of
// the data. Originating values that were valid will pass through the arithmetic and freezing
// exactly as they are; invalid values will become poison through the arithmetic and then be frozen
// to some unspecified value. The user may encounter garbage in such a case, but not in a way that
// triggers UB.
//
// Prior art and references
//
// https://github.com/rust-lang/rust/issues/21690
// Task for general purpose fast-math in rust lang. Discussions about the right approach
// and generalizability, including whether it should be type-based or annotation based. fast_fp
// uses types wrapping intrinsics because it's the only option available in user space, and gets
// good optimizations useful in practice
//
// https://docs.rs/fast-floats/0.2.0/fast_floats/index.html
// Another crate that wraps fast intrinsics in types. They didn't address poison propagation,
// leaving constructors unsafe
//
// https://llvm.org/docs/LangRef.html#fast-math-flags
// LLVM's documentation on fast-math
//
// https://llvm.org/docs/LangRef.html#poisonvalues
// LLVM's documentation on poison
//
// https://github.com/rust-lang/unsafe-code-guidelines/issues/71
// notes on the validity of primitive bit patterns

/// A wrapper over `f32` which enables some fast-math optimizations.
// TODO how best to document unspecified values, including witnessing possibly varying values
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FF32(MaybePoison<f32>);

/// Create a new `FF32` instance from the given float value.
///
/// This is syntax sugar for constructing the `FF32` type, and equivalent to `FF32::new(f)`
///
/// The given value **MUST NOT** be infinite or NaN, and any operations involving this value must
/// not produce infinite or NaN results. The output of any such operation is unspecified.
#[inline(always)]
pub fn ff32(f: f32) -> FF32 {
    // TODO maybe a feature flag to make this checked -> panic?
    FF32::new(f)
}

/// A wrapper over `f64` which enables some fast-math optimizations.
// TODO how best to document unspecified values, including witnessing possibly varying values
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FF64(MaybePoison<f64>);

/// Create a new `FF64` instance from the given float value.
///
/// This is syntax sugar for constructing the `FF64` type, and equivalent to `FF64::new(f)`
///
/// The given value **MUST NOT** be infinite or NaN, and any operations involving this value must
/// not produce infinite or NaN results. The output of any such operation is unspecified.
#[inline(always)]
pub fn ff64(f: f64) -> FF64 {
    // TODO maybe a feature flag to make this checked -> panic?
    FF64::new(f)
}

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
                    // Safety:
                    //
                    // - dereferencing the pointers is safe because every bit pattern is valid in float
                    // primitives
                    // - encountering poison operands is safe because LLVM's fast ops documents not producing
                    // UB on any inputs; it may produce poison on inf/nan (or if the sum is inf/nan), but these
                    // are then wrapped in the MaybePoison to control propagation
                    <$fast_ty>::new(unsafe {
                        $op_impl(
                            *self.0.maybe_poison().as_ptr(),
                            *other.0.maybe_poison().as_ptr(),
                        )
                    })
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

macro_rules! impl_assign_ops {
    ($fast_ty:ident, $base_ty: ident: $($op_trait:ident, $op_fn:ident, $op:ident,)*) => {
        $(
            impl $op_trait <$fast_ty> for $fast_ty {
                #[inline(always)]
                fn $op_fn(&mut self, rhs: $fast_ty) {
                    *self = <$fast_ty>::$op(*self, rhs)
                }
            }

            impl $op_trait <&$fast_ty> for $fast_ty {
                #[inline(always)]
                fn $op_fn(&mut self, rhs: &$fast_ty) {
                    *self = <$fast_ty>::$op(*self, rhs)
                }
            }

            impl $op_trait <$base_ty> for $fast_ty {
                #[inline(always)]
                fn $op_fn(&mut self, rhs: $base_ty) {
                    *self = <$fast_ty>::$op(*self, rhs)
                }
            }

            impl $op_trait <&$base_ty> for $fast_ty {
                #[inline(always)]
                fn $op_fn(&mut self, rhs: &$base_ty) {
                    *self = <$fast_ty>::$op(*self, rhs)
                }
            }
        )*
    }
}

macro_rules! impl_reduce_ops {
    ($fast_ty:ident, $base_ty: ident: $($op_trait:ident, $op_fn:ident, $op:ident, $identity:expr,)*) => {
        $(
            impl $op_trait <$fast_ty> for $fast_ty {
                #[inline]
                fn $op_fn <I> (iter: I) -> Self
                    where I: Iterator<Item = $fast_ty>
                {
                    iter.fold($identity, |acc, val| acc.$op(val))
                }
            }

            impl<'a> $op_trait <&'a $fast_ty> for $fast_ty {
                #[inline]
                fn $op_fn <I> (iter: I) -> Self
                    where I: Iterator<Item = &'a $fast_ty>
                {
                    iter.fold($identity, |acc, val| acc.$op(val))
                }
            }

            impl $op_trait <$base_ty> for $fast_ty {
                #[inline]
                fn $op_fn <I> (iter: I) -> Self
                    where I: Iterator<Item = $base_ty>
                {
                    iter.fold($identity, |acc, val| acc.$op(val))
                }
            }

            impl<'a> $op_trait <&'a $base_ty> for $fast_ty {
                #[inline]
                fn $op_fn <I> (iter: I) -> Self
                    where I: Iterator<Item = &'a $base_ty>
                {
                    iter.fold($identity, |acc, val| acc.$op(val))
                }
            }
        )*
    }
}

macro_rules! impl_fmt {
    ($fast_ty:ident, $base_ty:ident, $($fmt_trait:path,)*) => {
        $(
            impl $fmt_trait for $fast_ty {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    <$base_ty as $fmt_trait>::fmt(&self.freeze_raw(), f)
                }
            }
        )*
    }
}

#[cfg(feature = "num-traits")]
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

macro_rules! forward_freeze_self {
    ($fast_ty:ident, $base_ty:ident
     $(
         $(#[$attr:meta])*
         $vis:vis fn $fn_name:ident (self $(, $arg:ident : Self)* ) -> Self ;
     )*) => {
        $(
            $(#[$attr])*
            #[inline]
            $vis fn $fn_name(self $(, $arg : Self)*) -> Self {
                <$fast_ty>::new(<$base_ty>::$fn_name(self.freeze_raw() $(, $arg.freeze_raw())* ))
            }
        )*
     };
}

#[cfg(feature = "num-traits")]
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

#[cfg(feature = "num-traits")]
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
}

macro_rules! impls {
    ($fast_ty:ident, $base_ty: ident) => {
        impl $fast_ty {
            const ONE: $fast_ty = <$fast_ty>::new(1.0);
            const ZERO: $fast_ty = <$fast_ty>::new(0.0);

            #[doc = "Create a new `"]
            #[doc= stringify!($fast_ty)]
            #[doc = "` instance from the given float value."]
            ///
            /// The given value **MUST NOT** be infinite or NaN, and any operations involving this value must
            /// not produce infinite or NaN results. The output of any such operation is unspecified.
            #[inline(always)]
            pub const fn new(f: $base_ty) -> Self {
                $fast_ty(MaybePoison::new(f))
            }

            #[doc = "Create a new `"]
            #[doc= stringify!($fast_ty)]
            #[doc = "` instance from the given float value, returning an error if the value is infinite or NaN."]
            ///
            /// Note that this check is **not sufficient** to avoid all unspecified outputs, because an
            /// operation could otherwise produce an invalid value with valid inputs (for example
            /// `ff32(1.0) / ff32(0.0)` is unspecified). Nevertheless, this check can be useful for
            /// limited best-effort validation.
            #[inline(always)]
            pub fn new_checked(f: $base_ty) -> Result<Self, InvalidValueError> {
                // finite also checks for NaN
                if f.is_finite() {
                    Ok($fast_ty::new(f))
                } else {
                    Err(InvalidValueError { _priv: () })
                }
            }

            #[inline(always)]
            fn freeze_raw(self) -> $base_ty {
                let inner = self.0.freeze();

                // Safety:
                // every bit pattern is valid in float
                unsafe { inner.assume_init() }
            }

            // TODO migrate these to native implementations to freeze less and fast-math more
            forward_freeze_self! {
                $fast_ty, $base_ty
                pub fn abs(self) -> Self;
                pub fn acos(self) -> Self;
                pub fn acosh(self) -> Self;
                pub fn asin(self) -> Self;
                pub fn asinh(self) -> Self;
                pub fn atan(self) -> Self;
                pub fn atan2(self, other: Self) -> Self;
                pub fn atanh(self) -> Self;
                pub fn cbrt(self) -> Self;
                pub fn ceil(self) -> Self;
                pub fn clamp(self, min: Self, max: Self) -> Self;
                pub fn copysign(self, sign: Self) -> Self;
                pub fn cos(self) -> Self;
                pub fn cosh(self) -> Self;
                pub fn div_euclid(self, rhs: Self) -> Self;
                pub fn exp(self) -> Self;
                pub fn exp2(self) -> Self;
                pub fn exp_m1(self) -> Self;
                pub fn floor(self) -> Self;
                pub fn fract(self) -> Self;
                pub fn hypot(self, other: Self) -> Self;
                pub fn ln(self) -> Self;
                pub fn ln_1p(self) -> Self;
                pub fn log(self, base: Self) -> Self;
                pub fn log10(self) -> Self;
                pub fn log2(self) -> Self;
                pub fn max(self, other: Self) -> Self;
                pub fn min(self, other: Self) -> Self;
                pub fn mul_add(self, a: Self, b: Self) -> Self;
                pub fn powf(self, n: Self) -> Self;
                pub fn recip(self) -> Self;
                pub fn rem_euclid(self, rhs: Self) -> Self;
                pub fn round(self) -> Self;
                pub fn signum(self) -> Self;
                pub fn sin(self) -> Self;
                pub fn sinh(self) -> Self;
                pub fn sqrt(self) -> Self;
                pub fn tan(self) -> Self;
                pub fn tanh(self) -> Self;
                pub fn to_degrees(self) -> Self;
                pub fn to_radians(self) -> Self;
                pub fn trunc(self) -> Self;
            }

            #[inline]
            pub fn powi(self, n: i32) -> Self {
                <$fast_ty>::new(self.freeze_raw().powi(n))
            }

            #[inline]
            pub fn sin_cos(self) -> (Self, Self) {
                let (sin, cos) = self.freeze_raw().sin_cos();
                (<$fast_ty>::new(sin), <$fast_ty>::new(cos))
            }

            #[inline]
            pub fn classify(self) -> FpCategory {
                // NaN and infinity should not be presented as possibilities to users, even if
                // freeze ends up producing it. Results are unspecified, so Normal is just as valid
                // as any other answer
                match self.freeze_raw().classify() {
                    FpCategory::Nan | FpCategory::Infinite => FpCategory::Normal,
                    category => category
                }
            }

            #[inline]
            pub fn is_sign_negative(self) -> bool {
                // must freeze to keep poison out of bool branching
                self.freeze_raw().is_sign_negative()
            }

            #[inline]
            pub fn is_sign_positive(self) -> bool {
                // must freeze to keep poison out of bool branching
                self.freeze_raw().is_sign_positive()
            }

            #[inline]
            pub fn is_normal(self) -> bool {
                self.classify() == FpCategory::Normal
            }

            #[inline]
            pub fn is_subnormal(self) -> bool {
                self.classify() == FpCategory::Subnormal
            }

            /// The smallest finite value
            pub const MIN: $fast_ty = <$fast_ty>::new($base_ty::MIN);

            /// The smallest positive value
            pub const MIN_POSITIVE: $fast_ty = <$fast_ty>::new($base_ty::MIN_POSITIVE);

            /// The largest finite value
            pub const MAX: $fast_ty = <$fast_ty>::new($base_ty::MAX);
        }

        impl_fmt! {
            $fast_ty, $base_ty,
            fmt::Debug, fmt::Display, fmt::LowerExp, fmt::UpperExp,
        }

        impl_fast_ops! {
            $fast_ty, $base_ty:
            Add, add, fadd_fast,
            Sub, sub, fsub_fast,
            Mul, mul, fmul_fast,
            Div, div, fdiv_fast,
            Rem, rem, frem_fast,
        }

        impl_assign_ops! {
            $fast_ty, $base_ty:
            AddAssign, add_assign, add,
            SubAssign, sub_assign, sub,
            MulAssign, mul_assign, mul,
            DivAssign, div_assign, div,
            RemAssign, rem_assign, rem,
        }

        impl_reduce_ops! {
            $fast_ty, $base_ty:
            Sum, sum, add, Self::ZERO,
            Product, product, mul, Self::ONE,
        }

        impl Neg for $fast_ty {
            type Output = Self;

            #[inline(always)]
            fn neg(self) -> Self::Output {
                // Safety:
                //
                // - dereferencing the pointers is safe because every bit pattern is valid in float
                // primitives
                // - encountering poison is safe because LLVM's negate instruction documents
                // not producing UB on any inputs. The value is also immediately wrapped, so
                // poison propagation is controlled
                let val = unsafe { *self.0.maybe_poison().as_ptr() };
                $fast_ty::new(-val)
            }
        }

        impl Neg for &$fast_ty {
            type Output = <$fast_ty as Neg>::Output;

            #[inline]
            fn neg(self) -> Self::Output {
                -(*self)
            }
        }

        // Branching on poison values is UB, so any operation that makes a bool is protected by
        // freezing the operands. This includes [Partial]Eq and [Partial]Ord. Unfortunately
        // freezing has a nontrivial impact on performance, so non-bool methods should be preferred
        // when applicable, such as min/max/clamp
        //
        // Note however that only value copies are frozen; the original values may still be poison, and
        // could even yield different concrete values on a subsequent freeze. This means that potentially
        // the values are not Eq/Ord consistent. Logical consistency is left as a responsibility of
        // the user, to maintain non inf/nan values, while the lib only ensures safety.

        impl PartialEq<$fast_ty> for $fast_ty {
            #[inline]
            fn eq(&self, other: &$fast_ty) -> bool {
                let this = self.freeze_raw();
                let that = other.freeze_raw();

                this == that
            }
        }

        impl PartialEq<$base_ty> for $fast_ty {
            #[inline]
            fn eq(&self, other: &$base_ty) -> bool {
                let this = self.freeze_raw();
                let that = *other;

                this == that
            }
        }

        impl PartialEq<$fast_ty> for $base_ty {
            #[inline]
            fn eq(&self, other: &$fast_ty) -> bool {
                let this = *self;
                let that = other.freeze_raw();

                this == that
            }
        }

        impl Eq for $fast_ty {}

        impl PartialOrd<$fast_ty> for $fast_ty {
            #[inline(always)]
            fn partial_cmp(&self, other: &$fast_ty) -> Option<cmp::Ordering> {
                Some(self.cmp(other))
            }

            // TODO specialize a MaybePoison<bool> with `x & 0b1`?
            // then comparisons can freeze only once on output instead of twice on input

            #[inline(always)]
            fn lt(&self, other: &$fast_ty) -> bool {
                self.freeze_raw() < other.freeze_raw()
            }

            #[inline(always)]
            fn le(&self, other: &$fast_ty) -> bool {
                self.freeze_raw() <= other.freeze_raw()
            }

            #[inline(always)]
            fn gt(&self, other: &$fast_ty) -> bool {
                self.freeze_raw() > other.freeze_raw()
            }

            #[inline(always)]
            fn ge(&self, other: &$fast_ty) -> bool {
                self.freeze_raw() >= other.freeze_raw()
            }
        }

        impl Ord for $fast_ty {
            #[inline(always)]
            fn cmp(&self, other: &$fast_ty) -> cmp::Ordering {
                let this = self.freeze_raw();
                let that = other.freeze_raw();

                // Note NaNs are not supported (and would break everything else anyway) so we ignore them
                // and implement full Ord
                if this < that {
                    cmp::Ordering::Less
                } else if this > that {
                    cmp::Ordering::Greater
                } else {
                    cmp::Ordering::Equal
                }
            }

            #[inline]
            fn min(self, other: $fast_ty) -> $fast_ty {
                <$fast_ty>::min(self, other)
            }

            #[inline]
            fn max(self, other: $fast_ty) -> $fast_ty {
                <$fast_ty>::max(self, other)
            }

            #[inline]
            fn clamp(self, min: $fast_ty, max: $fast_ty) -> $fast_ty {
                <$fast_ty>::clamp(self, min, max)
            }
        }

        impl From<$fast_ty> for $base_ty {
            #[inline(always)]
            fn from(from: $fast_ty) -> Self {
                // base primitives are no longer in our API control, so we must stop poison
                // propagation by freezing
                from.freeze_raw()
            }
        }

        impl From<$base_ty> for $fast_ty {
            #[inline(always)]
            fn from(from: $base_ty) -> Self {
                <$fast_ty>::new(from)
            }
        }

        #[cfg(feature = "num-traits")]
        impl_num_traits! { $fast_ty, $base_ty }
    };
}

impls! { FF32, f32 }
impls! { FF64, f64 }

// TODO num_traits, libm?
