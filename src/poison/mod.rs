use core::mem::MaybeUninit;

/// A wrapper used to model LLVM's [poison
/// values](https://llvm.org/docs/LangRef.html#poisonvalues)
#[derive(Copy)]
#[repr(transparent)]
pub(crate) struct MaybePoison<T>(MaybeUninit<T>);

impl<T: Copy> Clone for MaybePoison<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> MaybePoison<T> {
    #[inline(always)]
    pub(crate) const fn new(t: T) -> Self {
        MaybePoison(MaybeUninit::new(t))
    }
}

/// A macro to implement poison handling *only* on types which are valid for every bit pattern
macro_rules! impl_maybe_poison {
    ($($raw_ty:ty),*) => {
        $(
            impl MaybePoison<$raw_ty> {
                /// Get the (possibly poison) value from this instance.
                ///
                /// The compiler may relax poison values to undefined values. That means, among other
                /// consequences, that calls to this function from copies of the same value could manifest
                /// different return values. Poison values are also transitive: an instruction that depends on
                /// a poison value, produces a poison value itself.
                ///
                /// Propogation of poison values can be stopped using [`freeze`](MaybePoison::freeze)
                ///
                /// # Safety
                ///
                /// It is UB to use a poison value as an operand to an instruction where _any_ of the operand's
                /// values trigger UB. This includes, for example, use as the divisor in integer division, or
                /// as the condition of a branch.
                ///
                /// See more examples and explanations in the [LLVM
                /// documentation](https://llvm.org/docs/LangRef.html#poisonvalues)
                #[inline(always)]
                pub(crate) unsafe fn maybe_poison(self) -> $raw_ty {
                    *self.0.as_ptr()
                }
            }
        )*
    }
}

macro_rules! impl_freeze {
    ($($raw_ty:ty, $fn_name:ident;)*) => {
        $(
            extern "C" {
                fn $fn_name(val: MaybePoison<$raw_ty>) -> $raw_ty;
            }

            impl MaybePoison<$raw_ty> {
                #[inline(always)]
                pub(crate) fn freeze(self) -> $raw_ty {
                    unsafe { $fn_name(self) }
                }
            }
        )*
    }
}

impl_maybe_poison! { f32, f64, u32, u64 }
impl_freeze! {
    f32, freeze_f32;
    f64, freeze_f64;
    //u32, freeze_i32;
    //u64, freeze_i64;
    //bool, freeze_i1;
}
