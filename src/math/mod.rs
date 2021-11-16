use crate::{poison::MaybePoison, FF32, FF64};
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

macro_rules! impl_extern_math {
    ($fast_ty:ident, $base_ty:ident) => {
        paste! {
            extern "C" {
                fn [<min_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
                fn [<max_ $base_ty>](a: $fast_ty, b: $fast_ty) -> $fast_ty;
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
            }
        }
    };
}

impl_generic_math! { FF32, f32, u32 }
impl_generic_math! { FF64, f64, u64 }

impl_extern_math! { FF32, f32 }
impl_extern_math! { FF64, f64 }
