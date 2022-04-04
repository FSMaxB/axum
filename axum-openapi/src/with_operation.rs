use crate::to_parameters::{Parameter, ToParameters};
use crate::to_responses::ToResponses;
use okapi::openapi3::{Components, Operation, Ref, RefOr};
use std::future::Future;

pub trait WithOperation<T> {
    type Type;

    fn split(self) -> (Self::Type, Operation, Components);
}

impl<F, Fut, Res> WithOperation<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Res>,
    Res: ToResponses,
{
    type Type = F;

    fn split(self) -> (Self::Type, Operation, Components) {
        let (responses, components) = Res::to_responses();
        let operation = Operation {
            responses,
            ..Default::default()
        };

        (self, operation, components)
    }
}

macro_rules! impl_with_operation_for_functions {
    ( $($type:ident), * $(,)?) => {
        impl<F, Fut, Res, $($type,)*> WithOperation<($($type,)*)> for F
        where
            F: FnOnce($($type,)*) -> Fut,
            Fut: Future<Output = Res>,
            Res: ToResponses,
            $( $type: ToParameters,)*
        {
            type Type = F;

            fn split(self) -> (Self::Type, Operation, Components) {
                let (responses, mut components) = Res::to_responses();
                let parameters = std::iter::empty()
                    $(
                        .chain($type::to_parameters())
                    )*
                    .map(|parameter| convert_parameter(parameter, &mut components))
                    .collect();
                let operation = Operation {
                    responses,
                    parameters,
                    ..Default::default()
                };

                (self, operation, components)
            }
        }
    }
}

all_the_tuples!(impl_with_operation_for_functions);

fn convert_parameter(
    parameter: Parameter,
    components: &mut Components,
) -> RefOr<okapi::openapi3::Parameter> {
    match parameter {
        Parameter::Parameter(parameter) => RefOr::Object(parameter),
        Parameter::Components(name, parameter) => {
            let reference = format!("#/components/parameters/{name}");
            components.parameters.insert(name, RefOr::Object(parameter));
            RefOr::Ref(Ref { reference })
        }
    }
}
