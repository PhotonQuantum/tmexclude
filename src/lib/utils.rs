#![allow(missing_docs)]

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
