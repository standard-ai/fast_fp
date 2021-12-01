#[cfg(any(feature = "nalgebra-v021", feature = "nalgebra-v029"))]
macro_rules! impl_nalgebra {
    ($($fast_ty:ident, $base_ty:ident),* ;
     $nalgebra_version:path, $simba_version:path, $approx_version:path ;
     @RealField: $real_field_callback:ident
     ) => {
        use $crate::{FF32, FF64, num_traits::forward_self};
        use $nalgebra_version as na;
        use $simba_version as simba;
        use $approx_version as approx;

        $(
            impl simba::simd::SimdValue for $fast_ty {
                type Element = Self;
                type SimdBool = bool;

                #[inline]
                fn lanes() -> usize {
                    1
                }

                #[inline]
                fn splat(val: Self::Element) -> Self {
                    val
                }

                #[inline]
                fn extract(&self, _:usize) -> Self::Element {
                    *self
                }

                #[inline]
                unsafe fn extract_unchecked(&self, _:usize) -> Self::Element {
                    *self
                }

                #[inline]
                fn replace(&mut self, _:usize, val: Self::Element) {
                    *self = val
                }

                #[inline]
                unsafe fn replace_unchecked(&mut self, _:usize, val: Self::Element) {
                    *self = val
                }

                #[inline]
                fn select(self, cond:Self::SimdBool, other: Self) -> Self {
                    if cond {
                        self
                    } else {
                        other
                    }
                }
            }

            impl simba::scalar::SubsetOf<f32> for $fast_ty {
                #[inline]
                fn to_superset(&self) -> f32 {
                    self.freeze_raw() as f32
                }

                #[inline]
                fn from_superset_unchecked(element: &f32) -> Self {
                    <$fast_ty>::new(*element as $base_ty)
                }

                #[inline]
                fn is_in_subset(_: &f32) -> bool {
                    true
                }
            }

            impl simba::scalar::SubsetOf<f64> for $fast_ty {
                #[inline]
                fn to_superset(&self) -> f64 {
                    self.freeze_raw() as f64
                }

                #[inline]
                fn from_superset_unchecked(element: &f64) -> Self {
                    <$fast_ty>::new(*element as $base_ty)
                }

                #[inline]
                fn is_in_subset(_: &f64) -> bool {
                    true
                }
            }

            impl simba::scalar::SubsetOf<FF32> for $fast_ty {
                #[inline]
                fn to_superset(&self) -> FF32 {
                    FF32::new(self.freeze_raw() as f32)
                }

                #[inline]
                fn from_superset_unchecked(element: &FF32) -> Self {
                    <$fast_ty>::new(element.freeze_raw() as $base_ty)
                }

                #[inline]
                fn is_in_subset(_: &FF32) -> bool {
                    true
                }
            }

            impl simba::scalar::SubsetOf<FF64> for $fast_ty {
                #[inline]
                fn to_superset(&self) -> FF64 {
                    FF64::new(self.freeze_raw() as f64)
                }

                #[inline]
                fn from_superset_unchecked(element: &FF64) -> Self {
                    <$fast_ty>::new(element.freeze_raw() as $base_ty)
                }

                #[inline]
                fn is_in_subset(_: &FF64) -> bool {
                    true
                }
            }

            impl simba::scalar::SubsetOf<$fast_ty> for f32 {
                #[inline]
                fn to_superset(&self) -> $fast_ty {
                    <$fast_ty>::new(*self as $base_ty)
                }

                #[inline]
                fn from_superset_unchecked(element: &$fast_ty) -> Self {
                    element.freeze_raw() as f32
                }

                #[inline]
                fn is_in_subset(_: &$fast_ty) -> bool {
                    true
                }
            }

            impl simba::scalar::SubsetOf<$fast_ty> for f64 {
                #[inline]
                fn to_superset(&self) -> $fast_ty {
                    <$fast_ty>::new(*self as $base_ty)
                }

                #[inline]
                fn from_superset_unchecked(element: &$fast_ty) -> Self {
                    element.freeze_raw() as f64
                }

                #[inline]
                fn is_in_subset(_: &$fast_ty) -> bool {
                    true
                }
            }

            impl approx::AbsDiffEq for $fast_ty {
                type Epsilon = Self;

                #[inline]
                fn default_epsilon() -> Self {
                    <$fast_ty>::new($base_ty::EPSILON)
                }

                #[inline]
                fn abs_diff_eq(&self, other: &Self, epsilon: Self) -> bool {
                    <$fast_ty>::abs(self - other) <= epsilon
                }
            }

            impl approx::UlpsEq for $fast_ty {
                #[inline]
                fn default_max_ulps() -> u32 {
                    <$base_ty>::default_max_ulps()
                }

                #[inline]
                fn ulps_eq(&self, other: &Self, epsilon: Self, max_ulps: u32) -> bool {
                    <$base_ty>::ulps_eq(
                        &self.freeze_raw(),
                        &other.freeze_raw(),
                        epsilon.freeze_raw(),
                        max_ulps
                    )
                }
            }

            impl approx::RelativeEq for $fast_ty {
                #[inline]
                fn default_max_relative() -> Self::Epsilon {
                    <$fast_ty as approx::AbsDiffEq>::default_epsilon()
                }

                #[inline]
                fn relative_eq(&self, other: &Self, epsilon: Self, max_relative: Self) -> bool {
                    <$base_ty>::relative_eq(
                        &self.freeze_raw(),
                        &other.freeze_raw(),
                        epsilon.freeze_raw(),
                        max_relative.freeze_raw(),
                    )
                }
            }

            impl na::Field for $fast_ty {}

            impl na::RealField for $fast_ty {
                forward_self! {
                    $fast_ty, $base_ty
                    fn atan2(self, other: Self) -> Self;
                    fn clamp(self, min: Self, max: Self) -> Self;
                    fn max(self, other: Self) -> Self;
                    fn min(self, other: Self) -> Self;
                }

                impl_nalgebra! {
                    @fn_consts $fast_ty, $base_ty
                    fn pi() -> Self ;
                    fn two_pi() -> Self ;
                    fn frac_pi_2() -> Self ;
                    fn frac_pi_3() -> Self ;
                    fn frac_pi_4() -> Self ;
                    fn frac_pi_6() -> Self ;
                    fn frac_pi_8() -> Self ;
                    fn frac_1_pi() -> Self ;
                    fn frac_2_pi() -> Self ;
                    fn frac_2_sqrt_pi() -> Self ;
                    fn e() -> Self ;
                    fn log2_e() -> Self ;
                    fn log10_e() -> Self ;
                    fn ln_2() -> Self ;
                    fn ln_10() -> Self ;
                }

                $real_field_callback! { $fast_ty, $base_ty }
            }

            impl na::ComplexField for $fast_ty {
                type RealField = Self;

                forward_self! {
                    $fast_ty, $base_ty
                    fn floor(self) -> Self;
                    fn ceil(self) -> Self;
                    fn round(self) -> Self;
                    fn trunc(self) -> Self;
                    fn fract(self) -> Self;
                    fn abs(self) -> Self;
                    fn signum(self) -> Self;
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
                    fn cbrt(self) -> Self;
                    fn hypot(self, other: Self) -> Self;
                    fn sin(self) -> Self;
                    fn cos(self) -> Self;
                    fn tan(self) -> Self;
                    fn asin(self) -> Self;
                    fn acos(self) -> Self;
                    fn atan(self) -> Self;
                    fn sin_cos(self) -> (Self, Self);
                    fn exp_m1(self) -> Self;
                    fn ln_1p(self) -> Self;
                    fn sinh(self) -> Self;
                    fn cosh(self) -> Self;
                    fn tanh(self) -> Self;
                    fn asinh(self) -> Self;
                    fn acosh(self) -> Self;
                    fn atanh(self) -> Self;
                }

                #[inline]
                fn from_real(re: Self::RealField) -> Self {
                    re
                }

                #[inline]
                fn real(self) -> Self::RealField {
                    self
                }

                #[inline]
                fn imaginary(self) -> Self::RealField {
                    Self::ZERO
                }

                #[inline]
                fn norm1(self) -> Self::RealField {
                    self.abs()
                }

                #[inline]
                fn modulus(self) -> Self::RealField {
                    self.abs()
                }

                #[inline]
                fn modulus_squared(self) -> Self::RealField {
                    self * self
                }

                #[inline]
                fn argument(self) -> Self::RealField {
                    if self >= Self::ZERO {
                        Self::ZERO
                    } else {
                        <$fast_ty>::new(core::$base_ty::consts::PI)
                    }
                }

                #[inline]
                fn scale(self, factor: Self::RealField) -> Self {
                    self * factor
                }

                #[inline]
                fn unscale(self, factor: Self::RealField) -> Self {
                    self / factor
                }

                #[inline]
                fn conjugate(self) -> Self {
                    self
                }

                #[inline]
                fn powc(self, n: Self) -> Self {
                    self.powf(n)
                }

                #[inline]
                fn is_finite(&self) -> bool {
                    true
                }

                #[inline]
                fn try_sqrt(self) -> Option<Self> {
                    let x = self.freeze_raw();
                    if x >= 0.0 {
                        Some(<$fast_ty>::new(x).sqrt())
                    } else {
                        None
                    }
                }
            }
        )*
    };

    (@fn_consts $fast_ty: ident, $base_ty: ident
        $(fn $fn_name:ident () -> Self ;)*
    ) => {
        $(
            fn $fn_name () -> $fast_ty {
                <$fast_ty>::new(<$base_ty>::$fn_name())
            }
        )*
    };
}

#[cfg(feature = "nalgebra-v021")]
#[cfg_attr(docsrs, doc(cfg(feature = "nalgebra-v021")))]
mod nalgebra_v021 {
    macro_rules! real_field {
        ($fast_ty:ident, $base_ty:ident) => {
            #[inline]
            fn is_sign_positive(self) -> bool {
                self > Self::ZERO
            }

            #[inline]
            fn is_sign_negative(self) -> bool {
                self < Self::ZERO
            }
        };
    }

    impl_nalgebra! {
        FF32, f32, FF64, f64 ;
        ::nalgebra_v021, ::simba_v01, ::approx_v03 ;
        @RealField: real_field
    }
}

#[cfg(feature = "nalgebra-v029")]
#[cfg_attr(docsrs, doc(cfg(feature = "nalgebra-v029")))]
mod nalgebra_v029 {
    macro_rules! real_field {
        ($fast_ty:ident, $base_ty:ident) => {
            #[inline]
            fn is_sign_positive(&self) -> bool {
                *self > Self::ZERO
            }

            #[inline]
            fn is_sign_negative(&self) -> bool {
                *self < Self::ZERO
            }

            forward_self! {
                $fast_ty, $base_ty
                fn copysign(self, other: Self) -> Self;
            }
        };
    }

    impl_nalgebra! {
        FF32, f32, FF64, f64 ;
        ::nalgebra_v029, ::simba_v06, ::approx_v05 ;
        @RealField: real_field
    }
}
