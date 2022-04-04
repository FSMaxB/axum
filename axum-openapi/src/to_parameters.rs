pub trait ToParameters {
    fn to_parameters() -> Vec<Parameter>;
}

pub enum Parameter {
    /// Use the parameter in-place
    Parameter(okapi::openapi3::Parameter),
    /// Put the parameter in the components and reference it.
    Components(String, okapi::openapi3::Parameter),
}

// FIXME: Parameters need to differentiate between "path", "query", "header", "cookie" and "requestContent"
// (the last one doesn't exist in OpenAPI parameters, but they are used as parameters in axum anyways.
