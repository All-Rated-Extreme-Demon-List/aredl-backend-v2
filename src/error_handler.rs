use actix_web::error::BlockingError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error_status_code: u16,
    pub error_message: String,
}

impl ApiError {
    pub fn new(error_status_code: u16, error_message: &str) -> ApiError {
        ApiError {
            error_status_code,
            error_message: error_message.to_string(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.error_message.as_str())
    }
}

impl From<DieselError> for ApiError {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::DatabaseError(_, err) => ApiError::new(409, err.message()),
            DieselError::NotFound => ApiError::new(404, "Record not found"),
            err => ApiError::new(500, &format!("Unknown Diesel error: {}", err)),
        }
    }
}

impl From<BlockingError> for ApiError {
    fn from(_error: BlockingError) -> Self {
        ApiError::new(500, "Internal server error")
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status_code = StatusCode::from_u16(self.error_status_code)
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR);

        let error_message = match status_code.as_u16() < 500 {
            true => self.error_message.clone(),
            false => "Internal server error".to_string(),
        };

        HttpResponse::build(status_code).json(json!({"message": error_message}))
    }
}
