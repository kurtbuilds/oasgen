#![allow(unused)]

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use actix_service::{Service, ServiceFactory};
use oasgen::{OaSchema, Server, openapi};
use actix_web::web::Json;
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Deserialize)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema, Debug)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

#[openapi]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse { found_account: false })
}

use actix_web::dev::ServiceResponse;
use actix_web::HttpResponse;
use actix_web::dev::ServiceRequest;
use actix_web::Responder;
use actix_web::FromRequest;
use actix_web::Handler;
use actix_service::always_ready;

#[derive(Debug, Clone)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Ready<T> {
    val: Option<T>,
}

impl<T> Ready<T> {
    /// Unwraps the value from this immediately ready future.
    #[inline]
    pub fn into_inner(mut self) -> T {
        self.val.take().unwrap()
    }
}

impl<T> Unpin for Ready<T> {}

impl<T> Future for Ready<T> {
    type Output = T;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        let val = self.val.take().expect("Ready can not be polled twice.");
        Poll::Ready(val)
    }
}

/// Creates a future that is immediately ready with a value.
#[allow(dead_code)]
pub(crate) fn ready<T>(val: T) -> Ready<T> {
    Ready { val: Some(val) }
}

/// Create a future that is immediately ready with a success value.
#[allow(dead_code)]
pub(crate) fn ok<T, E>(val: T) -> Ready<Result<T, E>> {
    Ready { val: Some(Ok(val)) }
}

pub struct FnService<F, Fut, Req, Res, Err>
    where
        F: FnMut(Req) -> Fut,
        Fut: Future<Output = Result<Res, Err>>,
{
    f: F,
    _t: PhantomData<fn(Req)>,
}

impl<F, Fut, Req, Res, Err> FnService<F, Fut, Req, Res, Err>
    where
        F: FnMut(Req) -> Fut,
        Fut: Future<Output = Result<Res, Err>>,
{
    pub(crate) fn new(f: F) -> Self {
        Self { f, _t: PhantomData }
    }
}

impl<F, Fut, Req, Res, Err> Clone for FnService<F, Fut, Req, Res, Err>
    where
        F: FnMut(Req) -> Fut + Clone,
        Fut: Future<Output = Result<Res, Err>>,
{
    fn clone(&self) -> Self {
        Self::new(self.f.clone())
    }
}

impl<F, Fut, Req, Res, Err> Service<Req> for FnService<F, Fut, Req, Res, Err>
    where
        F: Fn(Req) -> Fut,
        Fut: Future<Output = Result<Res, Err>>,
{
    type Response = Res;
    type Error = Err;
    type Future = Fut;

    always_ready!();

    fn call(&self, req: Req) -> Self::Future {
        (self.f)(req)
    }
}

pub struct FnServiceFactory<F, Fut, Req, Res, Err, Cfg>
    where
        F: Fn(Req) -> Fut,
        Fut: Future<Output = Result<Res, Err>>,
{
    f: F,
    _t: PhantomData<fn(Req, Cfg)>,
}

impl<F, Fut, Req, Res, Err, Cfg> FnServiceFactory<F, Fut, Req, Res, Err, Cfg>
    where
        F: Fn(Req) -> Fut + Clone,
        Fut: Future<Output = Result<Res, Err>>,
{
    fn new(f: F) -> Self {
        FnServiceFactory { f, _t: PhantomData }
    }
}

impl<F, Fut, Req, Res, Err, Cfg> Clone for FnServiceFactory<F, Fut, Req, Res, Err, Cfg>
    where
        F: Fn(Req) -> Fut + Clone,
        Fut: Future<Output = Result<Res, Err>>,
{
    fn clone(&self) -> Self {
        Self::new(self.f.clone())
    }
}

impl<F, Fut, Req, Res, Err> Service<Req> for FnServiceFactory<F, Fut, Req, Res, Err, ()>
    where
        F: Fn(Req) -> Fut + Clone,
        Fut: Future<Output = Result<Res, Err>>,
{
    type Response = Res;
    type Error = Err;
    type Future = Fut;

    always_ready!();

    fn call(&self, req: Req) -> Self::Future {
        (self.f)(req)
    }
}

impl<F, Fut, Req, Res, Err, Cfg> ServiceFactory<Req>
for FnServiceFactory<F, Fut, Req, Res, Err, Cfg>
    where
        F: Fn(Req) -> Fut + Clone,
        Fut: Future<Output = Result<Res, Err>>,
{
    type Response = Res;
    type Error = Err;

    type Config = Cfg;
    type Service = FnService<F, Fut, Req, Res, Err>;
    type InitError = ();
    type Future = Ready<Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: Cfg) -> Self::Future {
        ok(FnService::new(self.f.clone()))
    }
}

pub fn fn_service<F, Fut, Req, Res, Err, Cfg>(
    f: F,
) -> FnServiceFactory<F, Fut, Req, Res, Err, Cfg>
    where
        F: Fn(Req) -> Fut + Clone,
        Fut: Future<Output = Result<Res, Err>>,
{
    FnServiceFactory::new(f)
}

fn collector<F, Args>(handler: F)
    -> ()
where
    F: Handler<Args> + 'static,
    Args: FromRequest + 'static,
    F::Output: Responder + 'static,
{
    let z = move |req: ServiceRequest| {
        let handler = handler.clone();
        async move {
            let (req, mut payload) = req.into_parts();

            let res = match Args::from_request(&req, &mut payload).await {
                Err(err) => HttpResponse::from_error(err),

                Ok(data) => handler
                    .call(data)
                    .await
                    .respond_to(&req)
                    .map_into_boxed_body(),
            };

            Ok::<ServiceResponse, actix_web::Error>(ServiceResponse::new(req, res))
        }
    };
    let z: FnServiceFactory<_, _, _, _, _, ()> = fn_service(z);
    z.clone();
    // Box::new(z)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    use std::fs::File;
    use actix_web::{HttpResponse, web, HttpServer, App};
    // serde_yaml::to_writer(&File::create("examples/simple.yaml").unwrap(), &s.openapi).unwrap();

    let host = "0.0.0.0";
    let port = 5000;
    let host = format!("{}:{}", host, port);

    collector(send_code);

    // let server = Server::new()
    //     .post("/send-code", send_code)
    //     ;
        // .into_service();

    // HttpServer::new(move || App::new()
    // App::new()
    //     .route("/healthcheck", web::get().to(|| async { HttpResponse::Ok().body("Ok") }))
    //     .route("/send-code", web::post().to(send_code))
    // ;
    Ok(())
        // .service(server.clone().create_service())
        // .service(build_openapi().into_service("/api"))
        // .add_routes()
        // .wrap_api()
        // .route("/auth/send-code", post().to(auth::send_code))
                    // .with_json_spec_at("openapi.json")
                    // .build()
    // )
    //     .bind(host)?
    //     .run()
    //     .await
}