use okapi::openapi3::Operation;

pub trait WithOperation {
    type Type;

    fn split(self) -> (Self::Type, Operation);
}
