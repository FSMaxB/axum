use okapi::openapi3::Responses;

pub trait ToResponses {
    fn to_responses() -> Responses;
}
