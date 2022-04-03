use okapi::openapi3::{Components, Operation};

pub trait WithOperation {
    type Type;

    fn split(self) -> (Self::Type, Operation, Components);
}
