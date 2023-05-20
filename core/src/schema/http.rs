use http::{Method, Version, Uri};
use crate::OaSchema;

impl OaSchema for Method {}

impl OaSchema for Version {}

impl OaSchema for Uri {}
