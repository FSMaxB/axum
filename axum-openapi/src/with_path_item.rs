use okapi::openapi3::PathItem;

pub trait WithPathItem {
    type Type;

    fn split(self) -> (Self::Type, PathItem);
}
