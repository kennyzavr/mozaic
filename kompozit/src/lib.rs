use std::marker::PhantomData;

pub use kompozit_core::*;

#[doc(hidden)]
pub use kompozit_macros::compose;

pub use kompozit_macros::comp;
pub use kompozit_macros::comp_move;

pub fn from_fn<U: ?Sized, C: Composition<Unit = U>, O, F: FnOnce(&mut C) -> O>(
    f: F,
) -> impl Recomposition<Unit = U, Composition = C, Output = O> {
    struct Impl<C, O, F> {
        f: F,
        _comp: PhantomData<C>,
        _output: PhantomData<O>,
    }

    impl<C, O, F> Recomposition for Impl<C, O, F>
    where
        C: Composition,
        F: FnOnce(&mut C) -> O,
    {
        type Unit = <C as Composition>::Unit;
        type Composition = C;
        type Output = O;

        fn apply(self, composition: &mut Self::Composition) -> Self::Output {
            (self.f)(composition)
        }
    }

    return Impl {
        f,
        _comp: PhantomData,
        _output: PhantomData,
    };
}

pub mod private {
    pub use super::*;
    use ::core::marker::PhantomData;

    // compose ['comp'](::kompozit_core::Composition) composition to ['to'](::kompozit_core::Composition) composition.
    #[doc(hidden)]
    pub fn confirm_composition_possibility<C, T>(comp: &mut C, to: &mut T)
    where
        C: Composition<Unit = T::Unit>,
        T: Composition,
    {
    }

    pub trait Slot: From<Self::Source> {
        type Source;
        type Target;

        fn get(&mut self) -> &mut Self::Target;
    }

    pub struct Composer<Unit, Target, Stub>(StubComposer<Unit, Target, Stub>)
    where
        Unit: ?Sized,
        Stub: Composition<Unit = Unit>;

    pub struct StubComposer<Unit, Target, Stub>(
        PhantomData<Unit>,
        PhantomData<Target>,
        PhantomData<Stub>,
    )
    where
        Unit: ?Sized,
        Stub: Composition<Unit = Unit>;

    impl<Unit: ?Sized, Target, Stub: Composition<Unit = Unit>> ::core::ops::Deref
        for Composer<Unit, Target, Stub>
    {
        type Target = StubComposer<Unit, Target, Stub>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    pub fn composer<UnitProvider: Composition, Target>(
        _: &UnitProvider,
    ) -> Composer<UnitProvider::Unit, Target, StubComposition<UnitProvider::Unit>> {
        Composer(StubComposer(PhantomData, PhantomData, PhantomData))
    }

    impl<Unit: ?Sized, Target: Composition, Stub: Composition<Unit = Unit>>
        Composer<Unit, Target, Stub>
    {
        pub fn target_from_recomp<R: Recomposition<Composition = Target>>(&self, _: &R) -> &Self {
            self
        }
    }

    pub trait ComposeTarget<Target, Stub> {
        fn compose<'s>(
            &self,
            slot: &'s mut impl Slot<Target = Target>,
            default_slot: &mut Stub,
        ) -> &'s mut Target {
            println!("target");
            slot.get()
        }
    }

    pub trait ComposeStub<Target, Stub> {
        fn compose<'s>(
            &self,
            slot: &mut impl Slot<Target = Stub>,
            default_slot: &'s mut Target,
        ) -> &'s mut Target {
            println!("stub");
            default_slot
        }
    }

    impl<Unit: ?Sized, Target: Composition<Unit = Unit>, Stub: Composition<Unit = Unit>>
        ComposeTarget<Target, Stub> for &Composer<Unit, Target, Stub>
    {
    }

    impl<Unit: ?Sized, Target, Stub: Composition<Unit = Unit>> ComposeStub<Target, Stub>
        for Composer<Unit, Target, Stub>
    {
    }

    trait AreSame<A: ?Sized, B: ?Sized> {}
    impl<T: ?Sized> AreSame<T, T> for () {}
    trait HasUnit {
        type Unit: ?Sized;
    }
    impl<C: Composition> HasUnit for C {
        type Unit = <C as Composition>::Unit;
    }

    impl<Unit: ?Sized, Target, Stub: Composition<Unit = Unit>> Composer<Unit, Target, Stub> {
        pub fn check<C>(&self, _: &mut C)
        where
            C: HasUnit<Unit = Unit>,
            (): AreSame<C::Unit, Unit>,
        {
        }
    }

    pub enum NeverUnit {}

    pub trait NeverComposition: Composition<Unit = NeverUnit> {}
    impl<C: Composition<Unit = NeverUnit>> NeverComposition for C {}

    pub struct Caster<R>(PhantomData<R>);

    impl<R> Caster<R> {
        pub fn new(_: &R) -> Self {
            Self(PhantomData)
        }
    }

    pub trait CastNever<R: Recomposition<Unit = NeverUnit>>: Sized {
        fn cast<U: ?Sized>(&self, i: R) -> impl Recomposition<Unit = U, Output = R::Output> {
            let output = i.apply(&mut Composition::init());
            StubRecomposition(PhantomData, output)
        }
    }

    pub trait IsRecomp<R> {}
    impl<R: Recomposition> IsRecomp<R> for () {}

    pub fn check_to_recomp<R>(_: &R)
    where
        R: Recomposition,
    {
    }

    pub trait CastStub<R>: Sized {
        fn cast<U: ?Sized>(&self, i: R) -> impl Recomposition<Unit = U, Output = ()> {
            StubRecomposition(PhantomData, ())
        }
    }

    pub trait FallbackCastPrimary<C: Recomposition>: Sized {
        fn cast<T: ?Sized>(&self, i: C) -> C
        where
            C: Recomposition<Unit = T>,
        {
            i
        }
    }

    pub trait FallbackCastSecondary<C: Recomposition>: Sized {
        fn cast<T: ?Sized>(&self, i: C) -> C
        where
            C: Recomposition<Unit = T>,
        {
            i
        }
    }

    impl<R: Recomposition<Unit = NeverUnit>> CastNever<R> for &Caster<R> {}

    impl<R: Recomposition> FallbackCastPrimary<R> for &Caster<R> {}

    impl<R: Recomposition> FallbackCastSecondary<R> for Caster<R> {}

    impl<R> CastStub<R> for Caster<R> {}

    pub struct StubRecomposition<T: ?Sized, O>(PhantomData<T>, O);
    pub struct StubComposition<T: ?Sized>(PhantomData<T>);
    pub struct StubViewer<T: ?Sized>(PhantomData<T>);

    impl<T: ?Sized, O> StubRecomposition<T, O> {
        pub fn new(output: O) -> Self {
            Self(PhantomData, output)
        }
    }

    impl<T: ?Sized> Default for StubComposition<T> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    impl<T: ?Sized> Default for StubViewer<T> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    impl<Unit: ?Sized, Output> Recomposition for StubRecomposition<Unit, Output> {
        type Unit = Unit;
        type Composition = StubComposition<Unit>;
        type Output = Output;

        fn apply(self, composition: &mut Self::Composition) -> Self::Output {
            self.1
        }
    }

    impl<Unit: ?Sized> Composition for StubComposition<Unit> {
        type Unit = Unit;
        type Viewer<'s>
            = StubViewer<Self::Unit>
        where
            Self: 's;

        fn init() -> Self {
            Self(PhantomData)
        }

        fn view<'s>(&'s mut self) -> Self::Viewer<'s> {
            StubViewer(PhantomData)
        }
    }

    impl<Item: ?Sized> Viewer for StubViewer<Item> {
        type Item = Item;

        fn move_next(&mut self) {}

        fn move_prev(&mut self) {}

        fn current(&mut self) -> Option<&mut Self::Item> {
            None
        }
    }
}
