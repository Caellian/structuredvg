pub mod common;
pub mod error;
pub mod io;
pub mod math;
pub mod path;
pub mod script;
pub mod style;
pub mod svg;

pub(crate) mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T {}
}
