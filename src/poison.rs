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
    pub(crate) unsafe fn maybe_poison(self) -> MaybeUninit<T> {
        self.0
    }

    /// Freeze the poisoned value into a concrete (but arbitrary) value.
    ///
    /// Note that the value may not be a valid representation of T, so the return type is still
    /// unsafe to dereference unless T is valid with any representation.
    #[inline(always)]
    pub(crate) fn freeze(self) -> MaybeUninit<T> {
        // As of this writing, rust does not have any intrinsic to call LLVM's freeze instruction.
        // Instead, we do the next best thing by tricking the compiler into de-optimizing poison
        // values by introducing inline assembly. This is the same technique used by
        // `core::hint::black_box` and (the unmerged) https://github.com/rust-lang/rust/pull/58363.
        // We cannot use black_box directly, however, as it is documented as only a best-effort
        // hint, and could in theory be changed in the future.

        // Safety:
        //
        // - The poison value will no longer be poisoned, its safety restrictions no longer apply
        // - The asm macro emits no actual assembly, there's nothing to be unsafe
        unsafe {
            let inner = self.maybe_poison();
            // There is no actual assembly, it's just a trick to restrict the compiler from
            // optimizing around poison values. However the asm macro requires the format
            // string to capture all inputs, so put the captured pointer in an assembly comment.
            // The possibly poison value is labelled as input to the assembly block by providing a
            // pointer to the value; the compiler then must assume that anything could be done with
            // that pointer (e.g. reading and writing the value) so the compiler must materialize
            // a concrete (though arbitrary) value before the assembly
            asm!("/* {0} */", in(reg) inner.as_ptr(), options(nostack, preserves_flags));
            inner
        }
    }
}
