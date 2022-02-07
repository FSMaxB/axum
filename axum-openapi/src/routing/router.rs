use crate::with_path_item::{WithPathItem, WithPathItems};
use axum::body::{Body, HttpBody};
use axum::extract::connect_info::{Connected, IntoMakeServiceWithConnectInfo};
use axum::http::Request;
use axum::response::Response;
use axum::routing::future::RouteFuture;
use axum::routing::IntoMakeService;
use okapi::openapi3::PathItem;
use std::collections::HashMap;
use std::convert::Infallible;
use std::task::{Context, Poll};
use tower_service::Service;

#[derive(Clone, Debug)]
pub struct Router<B = Body> {
    router: axum::Router<B>,
    paths: HashMap<String, PathItem>,
}

impl<B: HttpBody + Send + 'static> Default for Router<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: HttpBody + Send + 'static> Service<Request<B>> for Router<B> {
    type Response = Response;
    type Error = Infallible;
    type Future = RouteFuture<B, Infallible>;

    fn poll_ready(&mut self, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.router.poll_ready(context)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        self.router.call(request)
    }
}

impl<B> WithPathItems for Router<B> {
    type Type = axum::routing::Router<B>;
    // FIXME: Don't expose internal HashMap type
    type PathItems = HashMap<String, PathItem>;

    fn split(self) -> (Self::Type, Self::PathItems) {
        (self.router, self.paths)
    }
}

impl<B> Router<B>
where
    B: HttpBody + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            router: axum::Router::new(),
            paths: Default::default(),
        }
    }

    pub fn route<T>(mut self, path: &str, service: T) -> Self
    where
        T: WithPathItem,
        T::Type:
            Service<Request<B>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        <T::Type as Service<Request<B>>>::Future: Send + 'static,
    {
        let (service, path_item) = service.split();
        self.router = self.router.route(path, service);
        self.paths.insert(path.to_owned(), path_item);

        self
    }

    pub fn nest<T>(mut self, path: &str, svc: T) -> Self
    where
        T: WithPathItems,
        T::Type:
            Service<Request<B>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        <T::Type as Service<Request<B>>>::Future: Send + 'static,
    {
        let (service, path_items) = svc.split();

        self.router = self.router.nest(path, service);

        for (nested_path, path_item) in path_items {
            let new_path = format!("{path}/{nested_path}");
            assert!(
                self.paths.insert(new_path.clone(), path_item).is_none(),
                "{} already exists",
                new_path
            );
        }

        self
    }

    pub fn merge(mut self, other: Router<B>) -> Self {
        // NOTE: This checks for overlap, so we don't need to to it
        self.router = self.router.merge(other.router);

        for (path, path_item) in other.paths {
            assert!(
                self.paths.insert(path.clone(), path_item).is_none(),
                "{} already exists",
                path
            );
        }

        self
    }

    // TODO: layer
    // TODO: route_layer

    pub fn fallback<T>(mut self, svc: T) -> Self
    where
        T: WithPathItems,
        T::Type:
            Service<Request<B>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        <T::Type as Service<Request<B>>>::Future: Send + 'static,
    {
        let (service, path_items) = svc.split();

        self.router = self.router.fallback(service);
        for (path, path_item) in path_items {
            assert!(
                self.paths.insert(path.clone(), path_item).is_none(),
                "{} already exists",
                path,
            );
        }

        self
    }

    pub fn into_make_service(self) -> IntoMakeService<axum::Router<B>> {
        self.router.into_make_service()
    }

    pub fn into_make_service_with_connect_info<C, Target>(
        self,
    ) -> IntoMakeServiceWithConnectInfo<axum::Router<B>, C>
    where
        C: Connected<Target>,
    {
        self.router.into_make_service_with_connect_info()
    }
}