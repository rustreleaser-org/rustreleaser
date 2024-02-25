use std::fmt::Display;

use thiserror::Error;

#[derive(Error, Debug)]
pub struct ErrorResponse {
    pub message: String,
    pub status: u16,
}

impl ErrorResponse {
    pub fn new(message: String, status: u16) -> Self {
        ErrorResponse { message, status }
    }

    pub fn internal_server_error(message: Option<impl Into<String>>) -> ErrorResponse {
        let message = if let Some(message) = message {
            message.into()
        } else {
            "Internal server error".to_string()
        };

        ErrorResponse {
            status: 500,
            message,
        }
    }
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Status: {}, Message: {}", self.status, self.message)
    }
}
