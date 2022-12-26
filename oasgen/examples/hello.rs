#![allow(unused)]

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use openapiv3::Operation;
use oasgen::OaSchema;
// use oasgen_core::OaOperation;
use pin_project_lite::pin_project;
use tokio;

pub trait OaOperation<Signature> {
    fn operation() -> Operation;
}


#[derive(OaSchema)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub phone: String,
    pub user_status: Option<i32>,
}

pin_project! {
    pub struct TypedResponseFuture<F, Signature> {
        #[pin]
        inner: F,
        _marker: std::marker::PhantomData<Signature>,
    }
}

impl<F, Signature> Future for TypedResponseFuture<F, Signature>
    where
        F: Future,
{
    type Output = F::Output;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        this.inner.poll(cx)
    }
}

impl<F, Signature> TypedResponseFuture<F, Signature>
{
    pub fn new(inner: F) -> Self {
        Self { inner, _marker: Default::default() }
    }
}


#[derive(OaSchema, Debug)]
pub struct SendCodeResponse {
    pub found_account: bool,
}


pub async fn send_code(mobile: String) -> SendCodeResponse {
    SendCodeResponse { found_account: false }
}

pub struct SendCodeSignature;

pub fn send_code_transformed(mobile: String) -> TypedResponseFuture<impl Future<Output=SendCodeResponse>, SendCodeSignature> {
    TypedResponseFuture { inner: send_code(mobile), _marker: Default::default() }
}

fn get_operation<Op, Signature>(operation: Op) -> Operation
    where
        Op: OaOperation<Signature>,
{
    Op::operation()
}

impl<F, A0, Fut> OaOperation<(A0, Fut)> for F
where
F: Fn(A0) -> TypedResponseFuture<Fut, SendCodeSignature>,
Fut: Future,
{
    fn operation() -> Operation {
        Operation::default()
    }
}

// impl<F, A, Fut, Output, Signature> OaOperation<(A, Fut, Output)> for F
//     where
//         F: Fn(A) -> TypedResponseFuture<Fut, Signature>,
//         // A: OaSchema,
//         // Output: OaSchema,
// {
//     fn operation() -> Operation {
//         F::operation()
//     }
// }


#[tokio::main]
async fn main() {
    let r = send_code_transformed("hi".to_string()).await;
    let op = get_operation(send_code_transformed);
    println!("{:?}", r);
    // println!("{:?}", op);
    // let s = User::schema();
    // println!("{:#?}", s);
}