use crate::{poison::MaybePoison, FF32, FF64};
use core::ops::{Add, Div, Mul, Rem, Sub};
use paste::paste;

impl FF32 {
    const SIGN_BIT: u32 = 0x8000_0000;
    const UNSIGNED_MASK: u32 = 0x7fff_ffff;
}

impl FF64 {
    const SIGN_BIT: u64 = 0x8000_0000_0000_0000;
    const UNSIGNED_MASK: u64 = 0x7fff_ffff_ffff_ffff;
}

macro_rules! impl_generic_math {
    ($fast_ty:ident, $base_ty:ident, $base_int:ident) => {
        impl $fast_ty {
            #[inline]
            fn to_bits(self) -> MaybePoison<$base_int> {
                // Safety:
                //
                // - `to_bits` should be valid for any input bits
                // - poison propagation is controlled with MaybePoison
                MaybePoison::new(unsafe { <$base_ty>::to_bits(self.0.maybe_poison()) })
            }

            #[inline]
            fn from_bits(bits: MaybePoison<$base_int>) -> Self {
                // Safety:
                //
                // - `from_bits` should be valid for any input bits
                // - poison propagation is controlled with MaybePoison
                Self(MaybePoison::new(unsafe {
                    <$base_ty>::from_bits(bits.maybe_poison())
                }))
            }

            #[inline]
            pub fn abs(self) -> Self {
                let bits = self.to_bits();
                <$fast_ty>::from_bits(MaybePoison::new(unsafe {
                    bits.maybe_poison() & Self::UNSIGNED_MASK
                }))
            }

            #[inline]
            pub fn copysign(self, other: Self) -> Self {
                let this = self.to_bits();
                let that = other.to_bits();

                // Safety:
                //
                // - & of poison is safe because & does not produce UB for any input values
                // - poison propagation is handled by wrapping in maybe poison
                <$fast_ty>::from_bits(MaybePoison::new(unsafe {
                    (this.maybe_poison() & Self::UNSIGNED_MASK)
                        | (that.maybe_poison() & Self::SIGN_BIT)
                }))
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
        }
    };
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

macro_rules! impl_extern_math {
    ($fast_ty:ident, $base_ty:ident) => {
        paste! {
            extern "C" {
                fn [<add_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
                fn [<sub_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
                fn [<mul_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
                fn [<div_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
                fn [<rem_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;

                fn [<min_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
                fn [<max_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;

                fn [<sqrt_ $base_ty>](a: $fast_ty) -> $fast_ty;
            }

            impl_fast_ops! {
                $fast_ty, $base_ty:
                Add, add, [<add_ $base_ty>],
                Sub, sub, [<sub_ $base_ty>],
                Mul, mul, [<mul_ $base_ty>],
                Div, div, [<div_ $base_ty>],
                Rem, rem, [<rem_ $base_ty>],
            }

            impl $fast_ty {
                #[inline]
                pub fn max(self, other: Self) -> Self {
                    unsafe { [<max_ $base_ty>](self, other) }
                }

                #[inline]
                pub fn min(self, other: Self) -> Self {
                    unsafe { [<min_ $base_ty>](self, other) }
                }

                #[inline]
                pub fn sqrt(self) -> Self {
                    unsafe { [<sqrt_ $base_ty>](self) }
                }
            }
        }
    };
}

impl_generic_math! { FF32, f32, u32 }
impl_generic_math! { FF64, f64, u64 }

impl_extern_math! { FF32, f32 }
impl_extern_math! { FF64, f64 }
