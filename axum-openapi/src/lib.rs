use okapi::openapi3::Components;

pub mod routing;
pub mod to_responses;
pub mod with_operation;
pub mod with_path_item;

pub(crate) trait ComponentExtensions {
    fn append(&mut self, other: &mut Components);
    fn merge(self, other: Components) -> Components;
}

impl ComponentExtensions for Components {
    fn append(&mut self, other: &mut Components) {
        let Components {
            schemas,
            responses,
            parameters,
            examples,
            request_bodies,
            headers,
            security_schemes,
            links,
            callbacks,
            extensions,
        } = self;

        schemas.append(&mut other.schemas);
        responses.append(&mut other.responses);
        parameters.append(&mut other.parameters);
        examples.append(&mut other.examples);
        request_bodies.append(&mut other.request_bodies);
        headers.append(&mut other.headers);
        security_schemes.append(&mut other.security_schemes);
        links.append(&mut other.links);
        callbacks.append(&mut other.callbacks);
        extensions.append(&mut other.extensions);
    }

    fn merge(mut self, mut other: Components) -> Components {
        self.append(&mut other);
        self
    }
}
