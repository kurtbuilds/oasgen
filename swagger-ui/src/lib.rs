mod config;

use std::convert::TryInto;
use std::error::Error;
use std::fmt::Debug;
pub use config::{Config, Url};
use rust_embed::RustEmbed;
use http::response::Response;

#[derive(RustEmbed)]
#[folder = "swagger-ui-dist"]
struct SwaggerUiDist;

#[derive(Debug, Clone)]
pub struct SwaggerUi {
    config: Config,
    prefix: String,
}

const HTML_MIME: &str = "text/html; charset=utf-8";
const JS_MIME: &str = "application/javascript; charset=utf-8";
const CSS_MIME: &str = "text/css; charset=utf-8";
const PNG_MIME: &str = "image/png";
const DEFAULT_MIME: &str = "application/octet-stream";

impl SwaggerUi {
    pub fn url<U: Into<Url>>(mut self, u: U) -> Self {
        self.config.url(u);
        self
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    pub fn handle_url<U>(&self, url: U) -> Option<Response<Vec<u8>>>
        where
            U: TryInto<http::Uri> + Debug,
            <U as TryInto<http::Uri>>::Error: Error
    {
        let url = url.try_into().unwrap();
        let path = url.path().strip_prefix(&self.prefix).unwrap();
        match path {
            "" | "/" => {
                let f = SwaggerUiDist::get("index.html").unwrap();
                let body = f.data.to_vec();
                Some(Response::builder()
                    .status(200)
                    .header("Content-Type", HTML_MIME)
                    .header("Content-Length", body.len())
                    .body(body)
                    .unwrap())
            }
            "/swagger-initializer.js" => {
                let f = SwaggerUiDist::get("swagger-initializer.js").unwrap();
                let body = String::from_utf8(f.data.to_vec()).unwrap();
                let config = serde_json::to_string(&self.config).unwrap();
                let body = body.replace("{config}", &config).into_bytes();
                Some(Response::builder()
                    .status(200)
                    .header("Content-Type", JS_MIME)
                    .header("Content-Length", body.len())
                    .body(body)
                    .unwrap())
            }
            z => {
                let f = SwaggerUiDist::get(&z[1..])?;
                let body = f.data.to_vec();
                let ext = std::path::Path::new(z).extension().unwrap().to_str().unwrap();
                let mime = match ext {
                    "html" => HTML_MIME,
                    "js" => JS_MIME,
                    "css" => CSS_MIME,
                    "png" => PNG_MIME,
                    _ => DEFAULT_MIME,
                };
                Some(Response::builder()
                    .status(200)
                    .header("Content-Type", mime)
                    .header("Content-Length", body.len())
                    .body(body)
                    .unwrap())
            }
        }
    }
}

impl Default for SwaggerUi {
    fn default() -> Self {
        Self {
            config: Config::default(),
            prefix: "".to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_swagger() {
        let ui = SwaggerUi::default();
        let res = ui.handle_url("/").unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers().get("Content-Type").unwrap(), HTML_MIME);

        let res = ui.handle_url("/swagger-initializer.js").unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers().get("Content-Type").unwrap(), JS_MIME);
        let body = String::from_utf8(res.body().to_vec()).unwrap();
        assert!(!body.contains("{config}"));

        let res = ui.handle_url("/swagger-ui.css").unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers().get("Content-Type").unwrap(), CSS_MIME);

        assert!(ui.handle_url("/not-found").is_none());
    }

    #[test]
    fn test_prefix_stripping() {
        let ui = SwaggerUi::default()
            .prefix("/docs");

        let res = ui.handle_url("/docs").unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers().get("Content-Type").unwrap(), HTML_MIME);

        let res = ui.handle_url("/docs/").unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers().get("Content-Type").unwrap(), HTML_MIME);

        let res = ui.handle_url("/docs/swagger-initializer.js").unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers().get("Content-Type").unwrap(), JS_MIME);

    }
}