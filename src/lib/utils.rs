#[allow(missing_docs)]
#[doc(hidden)]
pub trait TypeEq {
    type Rhs;
    fn cast(self) -> Self::Rhs;
}

impl<T> TypeEq for T {
    type Rhs = T;

    fn cast(self) -> Self::Rhs {
        self
    }
}
