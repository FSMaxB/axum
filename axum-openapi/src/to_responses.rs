use okapi::openapi3::{Components, Responses};

pub trait ToResponses {
    fn to_responses() -> (Responses, Components);
}
