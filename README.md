# Work in progress

# Try it working

```bash
ca run --example hello
```

# Development

```bash
cd oasgen
touch .env
mo RUSTFLAGS="-Z macro-backtrace"
jt
```

# Debugging

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
