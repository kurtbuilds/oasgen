<div id="top"></div>

<p align="center">
<a href="https://github.com/kurtbuilds/oasgen/graphs/contributors">
    <img src="https://img.shields.io/github/contributors/kurtbuilds/oasgen.svg?style=flat-square" alt="GitHub Contributors" />
</a>
<a href="https://github.com/kurtbuilds/oasgen/stargazers">
    <img src="https://img.shields.io/github/stars/kurtbuilds/oasgen.svg?style=flat-square" alt="Stars" />
</a>
<a href="https://github.com/kurtbuilds/oasgen/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/kurtbuilds/oasgen/test.yaml?style=flat-square" alt="Build Status" />
</a>
<a href="https://crates.io/crates/oasgen">
    <img src="https://img.shields.io/crates/d/oasgen?style=flat-square" alt="Downloads" />
</a>
<a href="https://crates.io/crates/oasgen">
    <img src="https://img.shields.io/crates/v/oasgen?style=flat-square" alt="Crates.io" />
</a>

</p>

# `oasgen` - OpenAPI Spec Generator

`oasgen` is a library to generate OpenAPI 3.0 specs from Rust server code (or any async functions). It supports:

- `actix` - actix-web
- `axum` - axum
- No framework - if you just want to register Rust functions to generate an OpenAPI spec file.

Contributions to support other web frameworks are welcome!

## Example

```rust
// Actix-web example
use actix_web::web::Json;
use actix_web::{App, HttpServer};
use oasgen::{
    actix::{get, scope},
    oasgen, OaSchema, Server,
};
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Deserialize)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema, Debug)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

#[oasgen]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse {
        found_account: false,
    })
}

#[oasgen]
async fn get_code() -> Json<String> {
    Json("code".into())
}

#[tokio::main]
async fn main() {
    HttpServer::new(move || {
        let server = Server::actix()
            .post("/send-code", send_code)
            .service(scope("/scoped").route("/get-code", get().to(get_code)))
            .freeze();
        App::new().service(server.into_service())
    })
    .bind(("127.0.0.1", 5000))
    .unwrap()
    .run()
    .await
    .unwrap()
}
```
To compile the actix-web example, use the following dependencies:

```toml
[dependencies]
actix-web = ".."
oasgen = { version = "..", features = ["actix", "actix-web"] }
serde = { version = "..", features = ["derive"] }
tokio = { version = "..", features = ["full"] }
```

```rust
// axum example
use oasgen::{OaSchema, Server, oasgen};
use axum::{Json, routing};
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Deserialize)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema, Debug)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

#[oasgen]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse { found_account: false })
}

#[tokio::main]
async fn main() {
    let server = Server::axum()
        .post("/send-code", send_code)
        .freeze();

    let router = axum::Router::new()
        .route("/healthcheck", routing::get(|| async { "OK" }))
        .merge(server.into_router());

    axum::Server::bind(&"0.0.0.0:5000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
```

To compile the axum example, use the following dependencies:

```toml
[dependencies]
axum = ".."
oasgen = { version = "..", features = ["axum"] }
serde = { version = "..", features = ["derive"] }
tokio = { version = "..", features = ["full"] }
```

# Installation

```toml
[dependencies]
# At minimum, you probably want a server feature installed (axum, actix) to support that framework
oasgen = { version = "..", features = []}
```

There are several features for activating other libraries:

- `actix` - actix-web
- `axum` - axum
- `swagger-ui` - swagger ui
- `uuid` - uuid
- `chrono` - chrono
- `time` - time
- `sqlx` - sqlx

# Customizing the generated spec

You can customize the generated spec in many ways.

## Direct access to the OpenAPI struct

You have direct access to the OpenAPI struct, so you can customize it however you want.

```rust
let mut server = Server::new();
server.openapi.info.title = "My API".to_string();
server.openapi.components.schemas.insert("MySchema".to_string(), Schema::new_object());
server
    .get("/my-route", my_handler)
    .freeze();

```

Note that you must make any changes before calling `.freeze()` (which moves the OpenAPI struct into an Arc to be shared between threads).

## Customizing a Schema

You can hand-write an implementation of OaSchema instead of using derive to customize any Schema. If you do this, call
`register_schema` after the struct definition to add it to the spec.

```rust
use oasgen::{OaSchema, Schema, register_schema};

pub struct User {
    pub id: i32,
    pub name: String,
}

impl OaSchema for User {
    fn schema() -> Schema {
        let mut schema = Schema::new_object();
        schema.properties_mut().insert("id", Schema::new_integer());
        schema.properties_mut().insert("name", Schema::new_string());
        schema
    }
}
register_schema!("User", &|| User::schema());
```

Technically speaking, you don't need to implement OaSchema at all.
You can pass any arbitrary closure that returns a `Schema` to the register_schema macro.

You can also customize an operation:

```rust
async fn my_server_handler() {
    // ...
}

// You must use the fully qualified path to the function.
// You can simplify this slightly by passing in `concat!(module_path!(), "::my_server_handler")`
register_operation!("my_server_crate::path::to::my_server_handler", &|| {
    let mut operation = Operation::default();
    operation.summary = Some("My summary".to_string());
    // ...
    operation
});
```

# Attributes

`oasgen` defines its own attributes, and also respects `serde` attributes. It also uses docstrings as descriptions.
You can see all attributes in `macro/src/attr.rs`. Look at those structs for relevant documentation, and see the examples below.

```rust

#[derive(OaSchema)]
pub struct User {
    pub id: i32,
    pub name: String,
    // Because oasgen respects serde attributes, this will not appear in the spec.
    #[serde(skip)]
    pub password_hash: String,
    // This will be in the response (because there's no serde(skip), but it will not show up in the OpenAPI spec.
    #[oasgen(skip)]
    pub internal_id: i32,
}

#[oasgen(
tags("auth", "users"),
summary = "This is a short summary"),
deprecated = true,
operation_id = "my_operation_id",
description = "This is a long description and will override the docstring of the function",
)]
async fn my_server_handler() {
    // ...
}
```

# Write the spec to a file

You have direct access to the `OpenAPI` struct. You can use `serde` to write it to a file, stdout, and more.

We provide a helper function `write_and_exit_if_env_var_set` that integrates well with a basic build process:

```rust
let server = Server::new()
    // your routes
    .write_and_exit_if_env_var_set("./openapi.yaml")
    // .freeze() here, if you're mounting to a server.
```

If `OASGEN_WRITE_SPEC=1`, it will write the spec to the path, then exit. 

In your build process, build the executable, run it once with the env var set to output the spec, then run it again without the env var 
to start the server normally.

# Route that displays the spec

> [!NOTE]  
> Requires the `swagger-ui` feature 

There are built-in functions to create routes that display the raw spec, or display a Swagger UI 
page for the spec.

```rust
let mut server = oasgen::Server::axum()
    .post("/auth/register_password", auth::register_password) // example route
    .route_yaml_spec("/openapi.yaml") // the spec will be available at /openapi.yaml
    .route_json_spec("/openapi.json") // the spec will be available at /openapi.json
    .swagger_ui("/openapi/"); // the swagger UI will be available at /openapi/.
                              // NOTE: The trailing slash is required, as is calling either `route_yaml_spec()` or `route_json_spec()` before `swagger_ui()`.
```

If you need to customize these routes, you have directly use a clone of the OpenAPI struct. It's in an Arc, so it's cheap to clone.

```rust
let mut server = oasgen::Server::axum()
    .post("/auth/register_password", auth::register_password) // example route
    .freeze();
let spec = server.openapi.clone();
let router = axum::Router::new()
    .merge(server.into_router())
    .route("/alt/route/to/openapi.yaml", get(|| {
        let spec = spec.clone();
        async { 
            serde_yaml::to_string(spec).unwrap()
        }
    }))
;
