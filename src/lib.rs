#![doc = include_str!("../README.md")]
#![feature(core_intrinsics)] // intrinsics for the fast math
#![feature(asm)] // asm used to emulate freeze
use core::{
    cmp, fmt,
    intrinsics::{fadd_fast, fdiv_fast, fmul_fast, frem_fast, fsub_fast},
    ops::{Add, Div, Mul, Neg, Rem, Sub},
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

/// A wrapper over `f32` which enables fast-math optimizations.
// TODO how best to document unspecified values, including witnessing possibly varying values
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FF32(MaybePoison<f32>);

impl FF32 {
    /// Create a new `FF32` instance from the given float value.
    ///
    /// The given value **MUST NOT** be infinite or NaN, and any operations involving this value must
    /// not produce infinite or NaN results. The output of any such operation is unspecified.
    #[inline(always)]
    pub const fn new(f: f32) -> Self {
        FF32(MaybePoison::new(f))
    }

    /// Create a new `FF32` instance from the given float value, returning an error if the value is
    /// infinite or NaN.
    ///
    /// Note that this check is **not sufficient** to avoid all unspecified outputs, because an
    /// operation could otherwise produce an invalid value with valid inputs (for example
    /// `ff32(1.0) / ff32(0.0)` is unspecified). Nevertheless, this check can be useful for
    /// limited best-effort validation.
    #[inline(always)]
    pub fn new_checked(f: f32) -> Result<Self, InvalidValueError> {
        // finite also checks for NaN
        if f.is_finite() {
            Ok(FF32::new(f))
        } else {
            Err(InvalidValueError { _priv: () })
        }
    }

    #[inline(always)]
    fn freeze_f32(self) -> f32 {
        let inner = self.0.freeze();

        // Safety:
        // every bit pattern is valid in float
        unsafe { inner.assume_init() }
    }
}

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

impl Neg for FF32 {
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
        FF32::new(-val)
    }
}

impl Neg for &FF32 {
    type Output = <FF32 as Neg>::Output;

    #[inline]
    fn neg(self) -> Self::Output {
        -(*self)
    }
}

impl fmt::Debug for FF32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.freeze_f32(), f)
    }
}

impl fmt::Display for FF32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.freeze_f32(), f)
    }
}

macro_rules! impl_refs {
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

            impl_refs! { $fast_ty, $fast_ty, $op_trait, $op_fn }
            impl_refs! { $fast_ty, $base_ty, $op_trait, $op_fn }
            impl_refs! { $base_ty, $fast_ty, $op_trait, $op_fn }
        )*
    };
}

impl_fast_ops! {
    FF32, f32:
    Add, add, fadd_fast,
    Sub, sub, fsub_fast,
    Mul, mul, fmul_fast,
    Div, div, fdiv_fast,
    Rem, rem, frem_fast,
}

// Branching on poison values is UB, so any operation that makes a bool is protected by freezing
// the operands. This includes [Partial]Eq and [Partial]Ord.
//
// Note however that only value copies are frozen; the original values may still be poison, and
// could even yield different concrete values on a subsequent freeze. This means that potentially
// the values are not Eq/Ord consistent. Logical consistency is left as a responsibility of
// the user, to maintain non inf/nan values, while the lib only ensures safety.

impl PartialEq<FF32> for FF32 {
    #[inline]
    fn eq(&self, other: &FF32) -> bool {
        let this = self.freeze_f32();
        let that = other.freeze_f32();

        this == that
    }
}

impl PartialEq<f32> for FF32 {
    #[inline]
    fn eq(&self, other: &f32) -> bool {
        let this = self.freeze_f32();
        let that = *other;

        this == that
    }
}

impl PartialEq<FF32> for f32 {
    #[inline]
    fn eq(&self, other: &FF32) -> bool {
        let this = *self;
        let that = other.freeze_f32();

        this == that
    }
}

impl Eq for FF32 {}

impl PartialOrd<FF32> for FF32 {
    #[inline(always)]
    fn partial_cmp(&self, other: &FF32) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }

    #[inline(always)]
    fn lt(&self, other: &FF32) -> bool {
        self.freeze_f32() < other.freeze_f32()
    }

    #[inline(always)]
    fn le(&self, other: &FF32) -> bool {
        self.freeze_f32() <= other.freeze_f32()
    }

    #[inline(always)]
    fn gt(&self, other: &FF32) -> bool {
        self.freeze_f32() > other.freeze_f32()
    }

    #[inline(always)]
    fn ge(&self, other: &FF32) -> bool {
        self.freeze_f32() >= other.freeze_f32()
    }
}

impl Ord for FF32 {
    #[inline(always)]
    fn cmp(&self, other: &FF32) -> cmp::Ordering {
        let this = self.freeze_f32();
        let that = other.freeze_f32();

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
    fn clamp(self, min: FF32, max: FF32) -> FF32 {
        ff32(f32::clamp(
            self.freeze_f32(),
            min.freeze_f32(),
            max.freeze_f32(),
        ))
    }
}

impl From<FF32> for f32 {
    fn from(from: FF32) -> Self {
        // f32 is no longer in our API control, so we must stop poison propagation by freezing
        from.freeze_f32()
    }
}

impl From<f32> for FF32 {
    fn from(from: f32) -> Self {
        ff32(from)
    }
}

// TODO FF64, macro everything, more ops, libm?
