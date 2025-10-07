use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use jsonschema::ErrorIterator;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use persistence::RepositoryError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub status_code: i32,

    #[serde(skip_deserializing)]
    pub trace_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub display_message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_message: Option<String>,

    #[serde(skip_deserializing, skip_serializing_if = "Vec::is_empty")]
    pub validation_errors: Vec<Error>,

    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub extended_messages: Option<BTreeMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    pub message: String,
    pub location: String,
}

#[derive(Debug, Clone)]
pub struct ResponseBuilder {
    response: ResponseMessage,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        ResponseBuilder {
            response: ResponseMessage::new(),
        }
    }

    pub fn status_code(&mut self, status_code: StatusCode) -> Self {
        self.response.status_code = status_code.as_u16() as i32;
        self.to_owned()
    }

    pub fn status_code_i64(&mut self, status_code: i64) -> Self {
        self.response.status_code = status_code as i32;
        self.to_owned()
    }

    pub fn error_message(&mut self, error_message: String) -> Self {
        self.response.error = Some(error_message);
        self.to_owned()
    }

    pub fn action_message(&mut self, action_message: String) -> Self {
        self.response.action_message = Some(action_message);
        self.to_owned()
    }

    pub fn description(&mut self, description: String) -> Self {
        self.response.description = Some(description);
        self.to_owned()
    }

    pub fn display_message(&mut self, display_message: String) -> Self {
        self.response.display_message = display_message;
        self.to_owned()
    }

    pub fn build(&self) -> ResponseMessage {
        self.response.to_owned()
    }
}

impl Default for ResponseMessage {
    fn default() -> Self {
        Self {
            status_code: 500,
            trace_id: String::default(),
            description: Some(String::from("Unexpected error in messages")),
            message: Some(String::from("Unexpected Error")),
            details: Default::default(),
            error: Default::default(),
            display_message: String::from("Ooops!!! An unexpected error has occured"),
            action_message: Some(String::from("You may try again or contact helpdesk")),
            validation_errors: Vec::default(),
            extended_messages: Default::default(),
        }
    }
}

impl From<ErrorIterator<'_>> for ResponseMessage {
    fn from(iter: ErrorIterator) -> Self {
        let mut res = ResponseMessage::bad_request();
        let val_errs: Vec<Error> = iter
            .map(|val_err| {
                let mut message = val_err.to_string();
                message = message.replace("\"", "");
                let mut location = val_err.instance_path.to_string();
                if location.len() == 0 {
                    location = "/".to_string()
                };
                Error { message, location }
            })
            .collect();

        res.validation_errors = val_errs;
        res
    }
}

impl From<RepositoryError> for ResponseMessage {
    fn from(repo_error: RepositoryError) -> Self {
        let mut err_response = ResponseMessage::default();
        err_response.error = Some(repo_error.to_string());
        err_response
    }
}

impl From<Value> for ResponseMessage {
    fn from(value: Value) -> Self {
        let err_response: ResponseMessage = match serde_json::from_value(value) {
            Ok(v) => v,
            Err(e) => {
                let mut default_err_msg = ResponseMessage::default();
                default_err_msg.extended_messages =
                    Some(BTreeMap::from([("root_error".to_string(), e.to_string())]));
                default_err_msg
            }
        };
        err_response
    }
}

/*impl From<Document> for ResponseMessage {
    fn from(doc: Document) -> Self {
        let err_response: ResponseMessage = bson::from_document(doc).unwrap();
        err_response
    }
}*/

impl Into<Value> for ResponseMessage {
    fn into(self) -> Value {
        return match serde_json::to_value(self) {
            Ok(val) => val,
            Err(e) => {
                let mut res = ResponseMessage::default();
                res.error = Some(e.to_string());
                res.into()
            }
        };
    }
}

impl IntoResponse for ResponseMessage {
    fn into_response(self) -> Response {
        let status_code = match StatusCode::from_u16(self.status_code as u16) {
            Ok(sc) => sc,
            Err(e) => {
                error!("Error getting status code {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (status_code, Json(self)).into_response()
    }
}

impl ResponseMessage {
    pub fn new() -> Self {
        ResponseMessage {
            status_code: 0,
            trace_id: String::default(),
            description: None,
            message: None,
            details: None,
            error: None,
            display_message: String::default(),
            action_message: None,
            validation_errors: vec![],
            extended_messages: None,
        }
    }

    pub fn with_status_code(&mut self, status_code: StatusCode) -> Self {
        self.status_code = status_code.as_u16() as i32;
        self.to_owned()
    }

    pub fn bad_request() -> Self {
        let mut response = ResponseMessage::default();
        response.description = Some("Validation Error(s) have occured".to_owned());
        response.message = Some("Validation Error".to_owned());
        response.status_code = 400;
        response.display_message = String::from("One or more validation errors have occured");
        response.action_message = Some(String::from("Please correct the errors and try again"));
        response
    }

    pub fn created() -> Self {
        let mut response = ResponseMessage::default();
        response.description = None;
        response.message = Some("Created".to_owned());
        response.status_code = 201;
        response.display_message = String::from("Operation succeeded");
        response.action_message = None;
        response
    }

    pub fn with_error(&mut self, err_msg: &str) -> Self {
        self.error = Some(String::from(err_msg));
        self.to_owned()
    }

    pub fn default_with_error(err_msg: &str) -> Self {
        let mut r = ResponseMessage::default();
        r.error = Some(String::from(err_msg));
        r
    }

    pub fn with_extended_message(&mut self, msg_key: String, msg_val: String) -> Self {
        match self.extended_messages {
            None => {
                self.extended_messages = Some(BTreeMap::<String, String>::new());
            }
            Some(_) => {}
        };
        self.extended_messages
            .as_mut()
            .unwrap()
            .insert(msg_key, msg_val);
        self.clone()
    }

    pub fn not_found() -> Self {
        Self {
            status_code: 404,
            trace_id: String::default(),
            description: Some(String::from("Not found")),
            message: Some(String::from("Resource is in hiding")),
            details: Default::default(),
            error: Default::default(),
            display_message: String::from(
                "Ooops!!! There is no resource matching the given criteria",
            ),
            action_message: Some(String::from("You may try again or contact helpdesk")),
            validation_errors: Vec::default(),
            extended_messages: Default::default(),
        }
    }
}
