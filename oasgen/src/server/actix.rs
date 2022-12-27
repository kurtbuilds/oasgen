use actix_web::{FromRequest, Handler, Responder};
use http::Method;
use oasgen_core::{OaOperation, OaSchema};
use super::Server;

pub trait MyClonableFn<'a> : Fn() -> MyNonCloneData {
    fn my_clone(&self) -> InnerRouteFactory<'static>;
}

impl<'a, T> MyClonableFn<'a> for T
where T: 'static + Clone + Fn() -> MyNonCloneData
{
    fn my_clone(&self) -> InnerRouteFactory<'static> {
        Box::new(self.clone())
    }
}

type MyNonCloneData = actix_web::Resource;
//
// pub(crate) trait MyCloneableFn<'a>: Fn() -> MyNonCloneData {
//     fn my_clone(&self) -> Box<dyn 'static + MyCloneableFn>;
// }
//
// // pub fn manual_clone<T: Clone + 'static>(vec: &Vec<Box<T>>) -> Vec<Box<T>> {
// //     vec.iter().map(|c| c.clone()).collect::<Vec<_>>()
// // }
//
// impl<'a, T> MyCloneableFn<'a> for T
//     where T: 'a + Clone + Fn() -> MyNonCloneData {
//     fn my_clone(&self) -> Box<dyn 'static + MyCloneableFn> {
//         Box::new(self.clone())
//     }
// }

pub type InnerRouteFactory<'a> = Box<dyn MyClonableFn<'a, Output=MyNonCloneData>>;

fn build_inner_resource<F, Args>(path: String, method: Method, handler: F) -> InnerRouteFactory<'static>
    where
        F: Handler<Args> + 'static + Copy,
        Args: FromRequest + 'static,
        F::Output: Responder + 'static,
{
    Box::new(move || {
        actix_web::Resource::new(path.clone())
            .route(actix_web::web::route().method(method.clone()).to(handler))
    })
}

impl Server {
    // #[cfg(feature = "actix")]
    pub fn get<F, Args, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: actix_web::Handler<Args> + OaOperation<Signature> + Copy,
            Args: actix_web::FromRequest + 'static,
            F::Output: actix_web::Responder + 'static,
            <F as actix_web::Handler<Args>>::Output: OaSchema,
    {
        self.update_spec(path, Method::GET, &handler);

        self.resources.push(build_inner_resource(path.to_string(), Method::GET, handler));
        self
    }

    // #[cfg(feature = "actix")]
    pub fn post<F, Args, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: actix_web::Handler<Args> + OaOperation<Signature> + Copy,
            Args: actix_web::FromRequest + 'static,
            F::Output: actix_web::Responder + 'static,
            <F as actix_web::Handler<Args>>::Output: OaSchema,
    {
        self.update_spec(path, Method::POST, &handler);

        self.resources.push(build_inner_resource(path.to_string(), Method::POST, handler));

        self
    }

    pub fn into_service(self) -> actix_web::Scope {
        let mut scope = actix_web::Scope::new("");
        for resource in self.resources {
            scope = scope.service(resource());
        }
        scope
    }
}