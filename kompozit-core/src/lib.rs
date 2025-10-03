pub trait Recomposition: Sized {
    type Unit: ?Sized;
    type Composition: Composition<Unit = Self::Unit>;
    type Output;

    fn apply(self, composition: &mut Self::Composition) -> Self::Output;
}

/// A composition: a container that holds *units* (e.g., UI Widgets)
/// and can produce a cursor-like [`Viewer`] to traverse and mutate them.
///
/// # Assosiated types
/// * [`Unit`] - the elementary item stored by this composition.
/// * [`Viewer`] - a cursor over the contained units that borrows the composition mutably.
///
/// # Lifetimes
/// `Viewer<'s>` borrows `self` mutably for `'s`. While a viewer is alive,
/// no other mutable access to the same composition is possible.
///
/// # Construction
/// Use ['init'] to create a fresh composition.
pub trait Composition {
    /// The unit stored by this composition (e.g. a widget node).
    ///
    /// # Sizing
    /// `Item` is allowed to be `?Sized`. This permits dynamicly sized types (e.g. `dyn Widget`)
    /// becouse [`Viewer::current`] returns mutable reference (`&mut Viewer::Item`) to the unit.
    type Unit: ?Sized;

    /// A cursor over the composed units.
    ///
    /// The viewer must borrow the composition mutably for `'s`.
    type Viewer<'s>: Viewer<Item = Self::Unit> + 's
    where
        Self: 's;

    /// Creates a new composition.
    fn init() -> Self;

    /// Returns the viewer over the contained units.
    ///
    /// The returned viewer holds a mutable borrow of the composition
    /// for a long as it lives.
    fn view<'s>(&'s mut self) -> Self::Viewer<'s>;
}

/// A cursor-like view over the units of a [`Composition`].
///
/// The viewer supports bidirectional movement and access to the current item.
pub trait Viewer {
    /// The type of unit being viewed.
    ///
    /// # Bounds
    /// The type of an item can be dynamicly sized becouse of
    /// [`Viewer::current`] method returns mutable reference to the item.
    type Item: ?Sized;

    /// Moves the cursor to the next item.
    ///
    /// If the cursor is already at the end, this method has no effect.
    fn move_next(&mut self);

    /// Moves the cursor to the previous item.
    ///
    /// If the cursor is already at the begining, this method has no effect.
    fn move_prev(&mut self);

    /// Returns a mutable reference to the current item or 'None' if the cursor
    /// is out of bounds (i.e. the composition is empty or movement has not positioned
    /// the cursor on a valid item).
    fn current(&mut self) -> Option<&mut Self::Item>;
}

impl<T: Composition> Composition for Option<T> {
    type Unit = T::Unit;
    type Viewer<'s>
        = Option<T::Viewer<'s>>
    where
        T: 's;

    fn init() -> Self {
        None
        // Some(<T as Composition>::init())
    }

    fn view<'s>(&'s mut self) -> Self::Viewer<'s> {
        self.as_mut().map(|c| c.view())
    }
}

impl<V: Viewer> Viewer for Option<V> {
    type Item = V::Item;

    fn move_next(&mut self) {
        if let Some(viewer) = self {
            viewer.move_next();
        }
    }

    fn move_prev(&mut self) {
        if let Some(viewer) = self {
            viewer.move_prev();
        }
    }

    fn current(&mut self) -> Option<&mut Self::Item> {
        self.as_mut().map(|v| v.current()).flatten()
    }
}
