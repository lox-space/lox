use lox_bodies::Body;

pub trait Origin {}

impl<U: Body> Origin for U {}

pub trait CoordinateOrigin<T: Origin> {
    fn origin(&self) -> T;
}
