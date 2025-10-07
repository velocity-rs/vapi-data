use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    sync::Arc,
};

use super::errors::StateError;
use axum::{extract::FromRequestParts, http::request::Parts};
use hashbrown::HashMap;
use jsonschema::{Draft, Validator};
use log::debug;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::LazyLock;
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<AppState>>;

/// Official SemVer regex
/// Source: https://semver.org/#is-there-a-suggested-regular-expression
/// Note: This regex is quite complex and may need to be adjusted based on specific requirements.
const SEMVER_REGEX: &str = r"
    ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|
    \d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?
    (?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$";

static SEMVER_REGEX_COMPILED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(SEMVER_REGEX).expect("Failed to compile SEMVER regex"));

#[derive(Debug, Clone)]
pub struct AppState {
    service_state: HashMap<ServicePath, ServiceState>,
    //root_state: HashMap<String, ServiceState>,
}

impl AppState {
    pub async fn new() -> Self {
        let service_state: HashMap<ServicePath, ServiceState> = HashMap::new();
        return AppState { service_state };
    }

    pub async fn get_service_state(&self, path: &ServicePath) -> Option<&ServiceState> {
        if let Some(service_state) = self.service_state.get(path) {
            return Some(service_state);
        }
        //check registry

        return None;
    }
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ServiceState {
    pub mongo_repo: persistence::Repository,
    pub validator: Arc<Validator>,
    pub schema: Value,
}

#[allow(dead_code)]
impl ServiceState {
    pub fn get_summmary(&self) -> String {
        let mut summary = String::default();
        summary.push_str("{ ");

        summary.push_str("Mongo Repo: [ ");
        summary.push_str(&self.mongo_repo.get_summary());

        summary.push_str("]. Compiled Schema: [ ");

        let valid = match self.validator.draft() {
            Draft::Draft202012 => true,
            _ => false,
        };
        summary.push_str(&("Valid draft: ".to_owned() + &valid.to_string()));
        summary.push_str("] }");

        summary.push_str(" }");
        summary
    }
}

/// This struct holds the path params from the request.
/// The service path is of the form /api/{namespace}/{name}/{version}/{optional_path_elements}
///
/// Therefore the path elements can be variable in length. The only fixed elements are the namespace, name and version.
/// The prefix is always /api/. But this will be changed to a configurable prefix in a later version.
///
/// The pattern is /api/{namespace}/{name}/{version}/...
/// Examples:
/// - /api/a/b/c/v1
/// - /api/a/b/c/d/v1
/// - /api/a/b/c/d/e/v1 etc
///
/// There could also be search params or such
///
/// - /api/a/b/c/v1?query=123
/// - /api/a/b/c/v1/search_def
///
/// TODO: Implement the configurable prefix.
///
/// The extractor will parse the path and extract the namespace, name and version. the implementation of FromRequestParts
/// allows the struct to be constructed from the request path using axum extractor mnechanics.
///
#[derive(Deserialize, Serialize, Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct ServicePath {
    pub namespace: String,
    pub name: String,
    pub version: String,
}

impl Display for ServicePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ Namespace: {}, Name: {}, Version: {} ]",
            self.namespace, self.name, self.version
        )
    }
}

impl<S> FromRequestParts<S> for ServicePath
where
    S: Send + Sync,
{
    type Rejection = StateError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let path = parts.uri.path();

        // Path takes the format /<prefix>/...<namespace>.../<name>/<version>/<optionals like search etc?
        //

        debug!("Extracting ServicePath from path: {}", path);
        let trimmed_path = path.trim_start_matches('/').trim_end_matches('/');
        let segments: Vec<&str> = trimmed_path.split('/').collect();

        if segments.len() < 4 || segments[0] != "api" {
            return Err(StateError::InvalidPath(path.to_string()));
        }

        // Check the index of the version segment
        // This means we have to check all segments matching regex

        todo!()
    }
}

impl FromStr for ServicePath {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();

        if parts.len() < 3 {
            return Err("Invalid service path format".to_string());
        }
        Ok(ServicePath {
            namespace: parts[0].to_string(),
            name: parts[1].to_string(),
            version: parts[2].to_string(),
        })
    }
}

#[allow(dead_code)]
impl ServicePath {
    pub fn new() -> Self {
        ServicePath::default()
    }

    pub fn get_namespace(&self) -> String {
        self.namespace.to_owned()
    }
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }
    pub fn get_version(&self) -> String {
        self.version.to_owned()
    }

    pub fn set_namespace(&mut self, org: String) -> Self {
        self.namespace = org;
        self.to_owned()
    }
    pub fn set_name(&mut self, app: String) -> Self {
        self.name = app;
        self.to_owned()
    }
    pub fn set_version(&mut self, version: String) -> Self {
        self.version = version;
        self.to_owned()
    }

    pub fn get_key(&self) -> String {
        let namespace = self.namespace.clone();
        let name = self.name.clone();
        let version = self.version.clone();

        let result = [namespace, name, version].join(":");
        debug!("Service Path Key {}", result);
        return result;
    }
}
