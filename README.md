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

`oasgen` is a library to generate OpenAPI 3.0 specs from Rust server code (or any async functions). We currently support:

- `actix` - actix-web
- No framework at all - if you just want to register Rust functions to generate an OpenAPI spec file.

Contributions to support other web frameworks are welcome!

## Example

```rust
// Actix-web example
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
        .freeze()
        ;
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
# For actix-web support
oasgen = { version = "0.1.0", features = ["actix"] }
```

# Debugging

Here are some issues you might encounter.

### Customizing the spec

You have direct access to the spec on its field: `oasgen::Server { pub openapi }`. You can modify it as you wish.

### Write the spec to a file

As noted, you have direct access to the `OpenAPI` struct. With it, you can write to stdout, the file, and more. 

We provide a helper function that integrates well with a basic build process.

```rust
let server = Server::new()
    // your routes
    .write_and_exit_if_env_var_set("./openapi.yaml")
    // .freeze() here, if you're mounting to a server.
```

If `OASGEN_WRITE_SPEC=1`, it will write the spec to the path, then exit. 

In your build process, build the executable, run it once with the env var set, then run it again without the env var 
to e.g. start the server normally.

### Route that displays the spec

It's easiest to use the built-in methods: `Schema::get_json_spec_at_path` and `Schema::get_yaml_spec_at_path`.

If you need to customize these routes, the closure lifetimes can be tricky. Below is a rough sketch.

```rust
async fn main() {
    let server = oasgen::Server::new();
    HttpServer::new(move || {
        let spec = server.openapi.clone();
        actix_web::App::new()
            .route("/openapi.json", web::get().to(move || {
                let spec = spec.clone();
                async { HttpResponse::Ok().json(&spec) }
            }))
            ;
    }) 
}
```

You can also assign a clone of the spec to a `web::Data` (or even a `lazy_static`!).

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
