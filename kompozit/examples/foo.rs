fn main() {
    let a = kompozit::from_fn(|scope: &mut kompozit::private::PhantomComposition<()>| {});
    let b = kompozit::from_fn(|scope: &mut kompozit::private::PhantomComposition<i32>| {});
    let c = kompozit::from_fn(|scope: &mut kompozit::private::PhantomComposition<()>| {});

    let c = kompozit::comp!({
        a.compose;
        c.compose;
    });

    fn foo(_: impl kompozit::Recomposition<Unit = i32>) {}
    // foo(c);
}
