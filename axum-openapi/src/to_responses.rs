use axum::Json;
use okapi::openapi3::{Components, MediaType, RefOr, Responses};
use okapi::schemars::gen::SchemaSettings;
use okapi::schemars::schema::{RootSchema, Schema};
use okapi::schemars::JsonSchema;
use okapi::{openapi3, schemars};

pub trait ToResponses {
    fn to_responses() -> (Responses, Components);
}

pub trait IntoResponse: ToResponses + axum::response::IntoResponse {}

impl<T> IntoResponse for T where T: ToResponses + axum::response::IntoResponse {}

impl ToResponses for () {
    fn to_responses() -> (Responses, Components) {
        (Responses::default(), Components::default())
    }
}

impl<T> ToResponses for Json<T>
where
    T: JsonSchema,
{
    fn to_responses() -> (Responses, Components) {
        let generator = SchemaSettings::openapi3().into_generator();

        let RootSchema {
            schema,
            definitions,
            ..
        } = generator.into_root_schema_for::<T>();

        let media_type = MediaType {
            schema: Some(schema),
            ..Default::default()
        };

        let definitions = definitions
            .into_iter()
            .filter_map(|(k, schema)| match schema {
                Schema::Bool(_) => None,
                Schema::Object(obj) => Some((k, obj)),
            })
            .collect::<schemars::Map<_, _>>();
        let mut components = Components::default();
        components.schemas.extend(definitions);

        let response = openapi3::Response {
            content: vec![("application/json".to_string(), media_type)]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let responses = Responses {
            default: Some(RefOr::Object(response)),
            responses: Default::default(),
            extensions: Default::default(),
        };

        (responses, components)
    }
}
