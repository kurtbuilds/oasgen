use serde::{Serialize};

const SWAGGER_STANDALONE_LAYOUT: &str = "StandaloneLayout";

#[non_exhaustive]
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Url to fetch external configuration from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_url: Option<String>,

    /// Id of the DOM element where `Swagger UI` will put it's user interface.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dom_id")]
    pub dom_id: Option<String>,

    /// [`Url`] the Swagger UI is serving.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Name of the primary url if any.
    #[serde(skip_serializing_if = "Option::is_none", rename = "urls.primaryName")]
    pub urls_primary_name: Option<String>,

    /// [`Url`]s the Swagger UI is serving.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub urls: Vec<Url>,

    /// Enables overriding configuration parameters with url query parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_config_enabled: Option<bool>,

    /// Controls whether [deep linking](https://github.com/swagger-api/swagger-ui/blob/master/docs/usage/deep-linking.md)
    /// is enabled in OpenAPI spec.
    ///
    /// Deep linking automatically scrolls and expands UI to given url fragment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deep_linking: Option<bool>,

    /// Controls whether operation id is shown in the operation list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_operation_id: Option<bool>,

    /// Default models expansion depth; -1 will completely hide the models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_models_expand_depth: Option<isize>,

    /// Default model expansion depth from model example section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model_expand_depth: Option<isize>,

    /// Defines how models is show when API is first rendered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model_rendering: Option<String>,

    /// Define whether request duration in milliseconds is displayed for "Try it out" requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_request_duration: Option<bool>,

    /// Controls default expansion for operations and tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_expansion: Option<String>,

    /// Defines is filtering of tagged operations allowed with edit box in top bar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<bool>,

    /// Controls how many tagged operations are shown. By default all operations are shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_displayed_tags: Option<usize>,

    /// Defines whether extensions are shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_extensions: Option<bool>,

    /// Defines whether common extensions are shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_common_extensions: Option<bool>,

    /// Defines whether "Try it out" section should be enabled by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub try_it_out_enabled: Option<bool>,

    /// Defines whether request snippets section is enabled. If disabled legacy curl snipped
    /// will be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_snippets_enabled: Option<bool>,

    /// Oauth redirect url.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2_redirect_url: Option<String>,

    /// Defines whether request mutated with `requestInterceptor` will be used to produce curl command
    /// in the UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_mutated_request: Option<bool>,

    /// Define supported http request submit methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_submit_methods: Option<Vec<String>>,

    /// Define validator url which is used to validate the Swagger spec. By default the validator swagger.io's
    /// online validator is used. Setting this to none will disable spec validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator_url: Option<String>,

    /// Enables passing credentials to CORS requests as defined
    /// [fetch standards](https://fetch.spec.whatwg.org/#credentials).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_credentials: Option<bool>,

    /// Defines whether authorizations is persisted throughout browser refresh and close.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persist_authorization: Option<bool>,

    /// The layout of Swagger UI uses, default is `"StandaloneLayout"`
    pub layout: String,
}

impl Config {
    pub fn url<U: Into<Url>>(&mut self, u: U) -> &mut Self {
        self.urls.push(u.into());
        self
    }
}


impl Default for Config {
    fn default() -> Self {
        Self {
            config_url: Default::default(),
            dom_id: Some("#swagger-ui".to_string()),
            url: Default::default(),
            urls_primary_name: Default::default(),
            urls: Default::default(),
            query_config_enabled: Default::default(),
            deep_linking: Some(true),
            display_operation_id: Default::default(),
            default_models_expand_depth: Default::default(),
            default_model_expand_depth: Default::default(),
            default_model_rendering: Default::default(),
            display_request_duration: Default::default(),
            doc_expansion: Default::default(),
            filter: Default::default(),
            max_displayed_tags: Default::default(),
            show_extensions: Default::default(),
            show_common_extensions: Default::default(),
            try_it_out_enabled: Default::default(),
            request_snippets_enabled: Default::default(),
            oauth2_redirect_url: Default::default(),
            show_mutated_request: Default::default(),
            supported_submit_methods: Default::default(),
            validator_url: Default::default(),
            with_credentials: Default::default(),
            persist_authorization: Default::default(),
            layout: SWAGGER_STANDALONE_LAYOUT.to_string(),
        }
    }
}



#[non_exhaustive]
#[derive(Default, Serialize, Clone, Debug)]
pub struct Url {
    name: String,
    url: String,
    #[serde(skip)]
    #[allow(dead_code)]
    primary: bool,
}

impl From<&str> for Url {
    fn from(url: &str) -> Self {
        Self {
            url: url.to_string(),
            ..Default::default()
        }
    }
}

impl From<String> for Url {
    fn from(url: String) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }
}

impl From<&String> for Url {
    fn from(url: &String) -> Self {
        Self {
            url: url.clone(),
            ..Default::default()
        }
    }
}