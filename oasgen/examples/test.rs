// use actix_web::{HttpServer, App, HttpResponse, web};
//
// #[tokio::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(move || {
//         App::new()
//             .route("/healthcheck", web::get().to(|| async {
//                 let s = "foo".to_string();
//                 HttpResponse::Ok().body(s)
//             }))
//             .service({
//                 let mut scope = web::scope("/api");
//                 scope = scope.service(web::resource("/foo").route(web::get().to(|| async { HttpResponse::Ok().body("foo") })));
//                 scope
//             })
//     })
//         .bind("0.0.0.0:5000")?
//         .run()
//         .await
// }
fn main() {

}