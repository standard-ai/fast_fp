#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::{
    cmp, fmt,
    iter::{Product, Sum},
    num::FpCategory,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign},
};

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

    ($fast_ty:ident, $base_ty:ident
     $(
         $(#[$attr:meta])*
         $vis:vis fn $fn_name:ident (&self $(, $arg:ident : &Self)* ) -> Self ;
     )*) => {
        $(
            $(#[$attr])*
            #[inline]
            $vis fn $fn_name(&self $(, $arg : &Self)*) -> Self {
                <$fast_ty>::new(<$base_ty>::$fn_name(self.freeze_raw() $(, $arg.freeze_raw())* ))
            }
        )*
    };
}

mod math;
mod nalgebra;
mod num_traits;

mod poison;
use poison::MaybePoison;

// The big challenge with fast-math in general is avoiding UB, and to a lesser extent unspecified
// values. LLVM's fast operations document "poison" behavior when given invalid inputs; poison
// values have a relatively consistent behavior (stuff like transitivity), defined cases for UB,
// and importantly can be limited in scope by freezing to a fixed value.
//
// This library manages these poison values to prevent UB. On the rust side, potentially-poison
// values are stored in a `MaybePoison` type, similar to the std's `MaybeUninit`. This helps ensure
// that the values would not trigger UB based on rust's semantics (for example, avoiding questions
// of whether all bit patterns of a primitive are valid). On the C side, operations are split into
// two groups: poison "safe" and poison "unsafe". Poison safe operations are ones which can accept
// any input value without triggering any UB. The operation may produce a poison value, for example
// `1.f / 0.f` with finite-math-only enabled, but not UB. Poison unsafe operations are ones which
// could trigger UB for some input value(s). These two definitions follow LLVM's documentation on
// poison, which explains poison can be relaxed to any value for a type, including `undef`.
// Therefore, if poison is passed to an operation it could be relaxed to any value; if some value
// could trigger UB, then so can poison.
//
// Poison safe operations are called with input values normally. They don't produce UB, so it's
// safe to call no matter the input. The operation is assumed to potentially produce poison itself,
// so the output is always wrapped in a `MaybePoison`.
//
// Poison unsafe operations must take certain precautions. First, any input arguments that are
// `MaybePoison` are frozen using LLVM's `freeze` instruction. This produces a value with an
// unspecified, but fixed, value which now won't be relaxed any further. Additionally, these
// operations are compiled without any flags that potentially introduce poison, regardless of
// enabled crate features. This ensures that the operation internally should not produce any poison
// regardless of input value. These two steps together preclude any poison values, which should
// prevent UB (assuming the operation was safe to call in the first place).
//
// All operations in rust are considered poison unsafe, and therefore must always freeze the value
// before using it. Freezing produces a regular f32/f64
//
// Prior art and references
//
// https://github.com/rust-lang/rust/issues/21690
// Task for general purpose fast-math in rust lang. Discussions about the right approach
// and generalizability, including whether it should be type-based or annotation based. fast_fp
// uses types because it's the only option available in user space, and gets good optimizations
// useful in practice
//
// https://docs.rs/fast-floats/0.2.0/fast_floats/index.html
// A crate that wraps fast intrinsics in types. Intrinsics only apply to basic ops, and they didn't
// address poison propagation, leaving constructors unsafe
//
// https://llvm.org/docs/LangRef.html#fast-math-flags
// LLVM's documentation on fast-math
//
// https://llvm.org/docs/LangRef.html#poisonvalues
// LLVM's documentation on poison
//
// https://github.com/rust-lang/unsafe-code-guidelines/issues/71
// notes on the validity of primitive bit patterns

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

macro_rules! impls {
    ($fast_ty:ident, $base_ty: ident) => {
        impl $fast_ty {
            const ONE: $fast_ty = <$fast_ty>::new(1.0);
            const ZERO: $fast_ty = <$fast_ty>::new(0.0);

            /// The smallest finite value
            pub const MIN: $fast_ty = <$fast_ty>::new($base_ty::MIN);

            /// The smallest positive value
            pub const MIN_POSITIVE: $fast_ty = <$fast_ty>::new($base_ty::MIN_POSITIVE);

            /// The largest finite value
            pub const MAX: $fast_ty = <$fast_ty>::new($base_ty::MAX);

            #[doc = "Create a new `"]
            #[doc = stringify!($fast_ty)]
            #[doc = "` instance from the given float value."]
            ///
            /// The given value **MUST NOT** be infinite or NaN, and any operations involving this value must
            /// not produce infinite or NaN results. The output of any such operation is unspecified.
            #[inline(always)]
            pub const fn new(f: $base_ty) -> Self {
                $fast_ty(MaybePoison::new(f))
            }

            #[doc = "Create a new `"]
            #[doc = stringify!($fast_ty)]
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
                self.0.freeze()
            }

            forward_freeze_self! {
                $fast_ty, $base_ty
                pub fn div_euclid(self, rhs: Self) -> Self;
                pub fn rem_euclid(self, rhs: Self) -> Self;
                pub fn to_degrees(self) -> Self;
                pub fn to_radians(self) -> Self;
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

            #[inline]
            pub fn hypot(self, other: Self) -> Self {
                (self * self + other * other).sqrt()
            }

            #[inline]
            pub fn signum(self) -> Self {
                Self::ONE.copysign(self)
            }

            #[inline]
            pub fn recip(self) -> Self {
                Self::ONE / self
            }

            #[inline]
            pub fn fract(self) -> Self {
                self - self.trunc()
            }

            #[inline]
            pub fn log(self, base: Self) -> Self {
                self.ln() / base.ln()
            }

            #[inline]
            pub fn mul_add(self, mul: Self, add: Self) -> Self {
                self * mul + add
            }

            #[inline]
            pub fn sin_cos(self) -> (Self, Self) {
                (self.sin(), self.cos())
            }
        }

        impl_fmt! {
            $fast_ty, $base_ty,
            fmt::Debug, fmt::Display, fmt::LowerExp, fmt::UpperExp,
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

        impl PartialOrd<$fast_ty> for $fast_ty {
            #[inline(always)]
            fn partial_cmp(&self, other: &$fast_ty) -> Option<cmp::Ordering> {
                <$base_ty>::partial_cmp(&self.freeze_raw(), &other.freeze_raw())
            }

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
    };
}

impls! { FF32, f32 }
impls! { FF64, f64 }
