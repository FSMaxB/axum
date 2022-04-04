use okapi::openapi3::{Components, Responses};

pub trait ToResponses {
    fn to_responses() -> (Responses, Components);
}

pub trait IntoResponse: ToResponses + axum::response::IntoResponse {}

impl<T> IntoResponse for T where T: ToResponses + axum::response::IntoResponse {}
