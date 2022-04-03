use okapi::openapi3::{Components, PathItem};

pub trait WithPathItem {
    type Type;

    fn split(self) -> (Self::Type, PathItem, Components);
}

pub trait WithPathItems {
    type Type;
    type PathItems: IntoIterator<Item = (String, PathItem)>;

    fn split(self) -> (Self::Type, Self::PathItems, Components);
}
