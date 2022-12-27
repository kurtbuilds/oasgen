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

`oasgen` is a library to generate OpenAPI 3.0 specs from Rust code.

This example shows actix-web (available behind an `actix` flag), but `oasgen` is not tied to any web framework, and
can also be used to register Rust functions into an OpenAPI spec without any web framework at all.

```rust
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server = Server::new()
        .post("/send-code", send_code)
        .;
    HttpServer::new(move || {
        App::new()
            .service(server.clone().into_service())
    })
        .bind("0.0.0.0:5000")?
        .run()
        .await 
}
```

# Installation

```toml
[dependencies]
oasgen = { version = "0.1.0", features = ["actix"] }
```

# Debugging

Here are some issues you might encounter.

#### I need to generate the spec at runtime


### Customizing the spec

You have direct access to the spec via `oasgen::Server { pub openapi }`, so you can do absolutely anything to customize it.

### Route to return the spec

It's easiest to use the built-in methods: `Schema::get_json_spec_at_path` and `Schema::get_yaml_spec_at_path`.

If you need to customize these routes, the closure lifetimes can be tricky. The solution looks like this:

```rust
async fn main() {
    let server = oasgen::Server::new();
    HttpServer::new(move || {
        let spec = server.openapi.clone();
        actix_web::App::new()
            .route("/openapi.json", web::get().to(move || {
                async { HttpResponse::Ok().json(&spec) }
            }))
            ;
    }) 
}
```

You can also assign a clone of the spec to a `web::Data`, or even a `lazy_static` to make it accessible elsewhere.

### Writing the spec to a file


### Support multiple web frameworks

Framework support is controlled via features, which are mutually incompatible. If you need to support
multiple frameworks in the same codebase, you can depend on `oasgen` multiple times with different
feature sets.

```toml
[dependencies]
oasgen_actix = { version = "0.1.0", features = ["actix"] , package = "oasgen" }
oasgen_none = { version = "0.1.0", package = "oasgen" }
```

Right now we only support actix, so this doesn't even apply, but the support exists for when 
other frameworks are needed.
