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

macro_rules! impl_freeze {
    ($($raw_ty:ty, $fn_name:ident;)*) => {
        $(
            #[link(name = "freeze")]
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

impl_freeze! {
    f32, freeze_f32;
    f64, freeze_f64;
}
