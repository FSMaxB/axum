pub trait ToParameters {
    fn to_parameters() -> Vec<Parameter>;
}

pub enum Parameter {
    /// Use the parameter in-place
    Parameter(okapi::openapi3::Parameter),
    /// Put the parameter in the components and reference it.
    Components(String, okapi::openapi3::Parameter),
}
