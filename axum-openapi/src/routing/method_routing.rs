use crate::with_operation::WithOperation;
use crate::with_path_item::WithPathItem;
use axum::body::{Body, Bytes, HttpBody};
use axum::handler::Handler;
use axum::http::Request;
use axum::response::Response;
use axum::routing::future::RouteFuture;
use axum::routing::MethodFilter;
use axum::BoxError;
use okapi::openapi3::{Operation, PathItem};
use std::convert::Infallible;
use std::task::{Context, Poll};
use tower_service::Service;

#[derive(Clone, Debug)]
pub struct MethodRouter<B = Body, E = Infallible> {
    method_router: axum::routing::MethodRouter<B, E>,
    method_operations: MethodOperations,
}

impl<B, E> WithPathItem for MethodRouter<B, E> {
    type Type = axum::routing::MethodRouter<B, E>;

    fn split(self) -> (Self::Type, PathItem) {
        (self.method_router, self.method_operations.into())
    }
}

#[derive(Default, Clone, Debug)]
struct MethodOperations {
    get: Option<Operation>,
    head: Option<Operation>,
    delete: Option<Operation>,
    options: Option<Operation>,
    patch: Option<Operation>,
    post: Option<Operation>,
    put: Option<Operation>,
    trace: Option<Operation>,
}

impl From<MethodOperations> for PathItem {
    fn from(method_operations: MethodOperations) -> Self {
        let MethodOperations {
            get,
            head,
            delete,
            options,
            patch,
            post,
            put,
            trace,
        } = method_operations;

        Self {
            get,
            put,
            post,
            delete,
            options,
            head,
            patch,
            trace,
            ..Default::default()
        }
    }
}

impl<B, E> Default for MethodRouter<B, E>
where
    B: Send + 'static,
{
    fn default() -> Self {
        Self {
            method_router: Default::default(),
            method_operations: Default::default(),
        }
    }
}

impl<B, E> Service<Request<B>> for MethodRouter<B, E>
where
    B: HttpBody,
{
    type Response = Response;
    type Error = E;
    type Future = RouteFuture<B, E>;

    fn poll_ready(&mut self, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.method_router.poll_ready(context)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        self.method_router.call(request)
    }
}

impl<B, E> MethodRouter<B, E> {
    pub fn new() -> Self {
        Self {
            method_router: axum::routing::MethodRouter::new(),
            method_operations: Default::default(),
        }
    }
}

macro_rules! top_level_operation {
    ($method:ident) => {
        pub fn $method<H, T, B>(handler: H) -> MethodRouter<B, Infallible>
        where
            H: WithOperation,
            H::Type: Handler<T, B>,
            B: Send + 'static,
            T: 'static,
        {
            MethodRouter::new().$method(handler)
        }
    };
}

macro_rules! operation {
    ($method:ident) => {
        pub fn $method<H, T>(mut self, handler: H) -> Self
        where
            H: WithOperation,
            H::Type: Handler<T, B>,
            T: 'static,
        {
            let (handler, operation) = handler.split();

            self.method_operations.$method = Some(operation);
            self.method_router = self.method_router.$method(handler);
            self
        }
    };
}

pub fn any<H, T, B>(handler: H) -> MethodRouter<B, Infallible>
where
    H: WithOperation,
    H::Type: Handler<T, B>,
    B: Send + 'static,
    T: 'static,
{
    let (handler, operation) = handler.split();

    MethodRouter {
        method_router: axum::routing::any(handler),
        method_operations: MethodOperations {
            get: Some(operation.clone()),
            head: Some(operation.clone()),
            delete: Some(operation.clone()),
            options: Some(operation.clone()),
            patch: Some(operation.clone()),
            post: Some(operation.clone()),
            put: Some(operation.clone()),
            trace: Some(operation),
        },
    }
}

pub fn on<H, T, B>(filter: MethodFilter, handler: H) -> MethodRouter<B, Infallible>
where
    H: WithOperation,
    H::Type: Handler<T, B>,
    B: Send + 'static,
    T: 'static,
{
    MethodRouter::new().on(filter, handler)
}

top_level_operation!(get);
top_level_operation!(head);
top_level_operation!(delete);
top_level_operation!(options);
top_level_operation!(patch);
top_level_operation!(post);
top_level_operation!(put);
top_level_operation!(trace);

impl<B> MethodRouter<B, Infallible>
where
    B: Send + 'static,
{
    operation!(get);
    operation!(head);
    operation!(delete);
    operation!(options);
    operation!(patch);
    operation!(post);
    operation!(put);
    operation!(trace);

    pub fn on<H, T>(mut self, filter: MethodFilter, handler: H) -> Self
    where
        H: WithOperation,
        H::Type: Handler<T, B>,
        T: 'static,
    {
        let (handler, operation) = handler.split();

        if filter.contains(MethodFilter::GET) {
            self.method_operations.get = Some(operation.clone());
        }
        if filter.contains(MethodFilter::HEAD) {
            self.method_operations.head = Some(operation.clone());
        }
        if filter.contains(MethodFilter::DELETE) {
            self.method_operations.delete = Some(operation.clone());
        }
        if filter.contains(MethodFilter::OPTIONS) {
            self.method_operations.options = Some(operation.clone());
        }
        if filter.contains(MethodFilter::PATCH) {
            self.method_operations.patch = Some(operation.clone());
        }
        if filter.contains(MethodFilter::POST) {
            self.method_operations.post = Some(operation.clone());
        }
        if filter.contains(MethodFilter::PUT) {
            self.method_operations.put = Some(operation.clone());
        }
        if filter.contains(MethodFilter::TRACE) {
            self.method_operations.trace = Some(operation);
        }

        self.method_router = self.method_router.on(filter, handler);
        self
    }
}

macro_rules! top_level_service_operation {
    ($method:ident, $method_service:ident) => {
        pub fn $method_service<S, ReqBody, ResBody>(
            svc: S,
        ) -> MethodRouter<ReqBody, <S::Type as Service<Request<ReqBody>>>::Error>
        where
            S: WithOperation,
            S::Type:
                Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
            <S::Type as Service<Request<ReqBody>>>::Future: Send + 'static,
            ResBody: HttpBody<Data = Bytes> + Send + 'static,
            ResBody::Error: Into<BoxError>,
        {
            MethodRouter::new().$method_service(svc)
        }
    };
}

macro_rules! service_operation {
    ($method:ident, $method_service:ident) => {
        pub fn $method_service<S, ResBody>(mut self, svc: S) -> Self
        where
            S: WithOperation,
            S::Type: Service<Request<ReqBody>, Response = Response<ResBody>, Error = E>
                + Clone
                + Send
                + 'static,
            <S::Type as Service<Request<ReqBody>>>::Future: Send + 'static,
            ResBody: HttpBody<Data = Bytes> + Send + 'static,
            ResBody::Error: Into<BoxError>,
        {
            let (service, operation) = svc.split();

            self.method_operations.$method = Some(operation);
            self.method_router = self.method_router.$method_service(service);
            self
        }
    };
}

pub fn any_service<S, ReqBody, ResBody>(
    svc: S,
) -> MethodRouter<ReqBody, <S::Type as Service<Request<ReqBody>>>::Error>
where
    S: WithOperation,
    S::Type: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    <S::Type as Service<Request<ReqBody>>>::Future: Send + 'static,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError>,
{
    MethodRouter::new().fallback(svc)
}

pub fn on_service<S, ReqBody, ResBody>(
    filter: MethodFilter,
    svc: S,
) -> MethodRouter<ReqBody, <S::Type as Service<Request<ReqBody>>>::Error>
where
    S: WithOperation,
    S::Type: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    <S::Type as Service<Request<ReqBody>>>::Future: Send + 'static,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError>,
{
    MethodRouter::new().on_service(filter, svc)
}

top_level_service_operation!(get, get_service);
top_level_service_operation!(head, head_service);
top_level_service_operation!(delete, delete_service);
top_level_service_operation!(options, options_service);
top_level_service_operation!(patch, patch_service);
top_level_service_operation!(post, post_service);
top_level_service_operation!(put, put_service);
top_level_service_operation!(trace, trace_service);

impl<ReqBody, E> MethodRouter<ReqBody, E> {
    service_operation!(get, get_service);
    service_operation!(head, head_service);
    service_operation!(delete, delete_service);
    service_operation!(options, options_service);
    service_operation!(patch, patch_service);
    service_operation!(post, post_service);
    service_operation!(put, put_service);
    service_operation!(trace, trace_service);

    pub fn on_service<S, ResBody>(mut self, filter: MethodFilter, svc: S) -> Self
    where
        S: WithOperation,
        S::Type: Service<Request<ReqBody>, Response = Response<ResBody>, Error = E>
            + Clone
            + Send
            + 'static,
        <S::Type as Service<Request<ReqBody>>>::Future: Send + 'static,
        ResBody: HttpBody<Data = Bytes> + Send + 'static,
        ResBody::Error: Into<BoxError>,
    {
        let (service, operation) = svc.split();

        if filter.contains(MethodFilter::GET) {
            self.method_operations.get = Some(operation.clone());
        }
        if filter.contains(MethodFilter::HEAD) {
            self.method_operations.head = Some(operation.clone());
        }
        if filter.contains(MethodFilter::DELETE) {
            self.method_operations.delete = Some(operation.clone());
        }
        if filter.contains(MethodFilter::OPTIONS) {
            self.method_operations.options = Some(operation.clone());
        }
        if filter.contains(MethodFilter::PATCH) {
            self.method_operations.patch = Some(operation.clone());
        }
        if filter.contains(MethodFilter::POST) {
            self.method_operations.post = Some(operation.clone());
        }
        if filter.contains(MethodFilter::PUT) {
            self.method_operations.put = Some(operation.clone());
        }
        if filter.contains(MethodFilter::TRACE) {
            self.method_operations.trace = Some(operation);
        }

        self.method_router = self.method_router.on_service(filter, service);
        self
    }

    pub fn fallback<S, ResBody>(mut self, svc: S) -> Self
    where
        S: WithOperation,
        S::Type: Service<Request<ReqBody>, Response = Response<ResBody>, Error = E>
            + Clone
            + Send
            + 'static,
        <S::Type as Service<Request<ReqBody>>>::Future: Send + 'static,
        ResBody: HttpBody<Data = Bytes> + Send + 'static,
        ResBody::Error: Into<BoxError>,
    {
        let (service, operation) = svc.split();

        if self.method_operations.get.is_none() {
            self.method_operations.get = Some(operation.clone());
        }
        if self.method_operations.head.is_none() {
            self.method_operations.head = Some(operation.clone());
        }
        if self.method_operations.delete.is_none() {
            self.method_operations.delete = Some(operation.clone());
        }
        if self.method_operations.options.is_none() {
            self.method_operations.options = Some(operation.clone());
        }
        if self.method_operations.patch.is_none() {
            self.method_operations.patch = Some(operation.clone());
        }
        if self.method_operations.post.is_none() {
            self.method_operations.post = Some(operation.clone());
        }
        if self.method_operations.put.is_none() {
            self.method_operations.put = Some(operation.clone());
        }
        if self.method_operations.trace.is_none() {
            self.method_operations.trace = Some(operation);
        }

        self.method_router = self.method_router.fallback(service);
        self
    }

    // TODO: layer
    // TODO: route_layer
    // TODO: handle_error

    pub fn merge(self, other: MethodRouter<ReqBody, E>) -> Self {
        let Self {
            method_router: our_method_router,
            method_operations:
                MethodOperations {
                    get: our_get,
                    put: our_put,
                    post: our_post,
                    delete: our_delete,
                    options: our_options,
                    head: our_head,
                    patch: our_patch,
                    trace: our_trace,
                },
        } = self;
        let Self {
            method_router: other_method_router,
            method_operations:
                MethodOperations {
                    get: other_get,
                    put: other_put,
                    post: other_post,
                    delete: other_delete,
                    options: other_options,
                    head: other_head,
                    patch: other_patch,
                    trace: other_trace,
                },
        } = other;

        Self {
            // This also verifies that the methods don't overlap.
            method_router: our_method_router.merge(other_method_router),
            method_operations: MethodOperations {
                get: our_get.or(other_get),
                put: our_put.or(other_put),
                post: our_post.or(other_post),
                delete: our_delete.or(other_delete),
                options: our_options.or(other_options),
                head: our_head.or(other_head),
                patch: our_patch.or(other_patch),
                trace: our_trace.or(other_trace),
            },
        }
    }
}