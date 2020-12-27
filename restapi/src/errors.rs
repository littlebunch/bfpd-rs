use actix_web::http::StatusCode;

use crate::routes::MAX_RECS;
use serde::Serialize;
#[derive(Debug)]
#[allow(dead_code)]
pub enum CustomError {
    MaxValidationError,
    OrderError,
    OffsetError,
    FoodSortError,
    Unknown,
}
#[derive(Serialize)]
pub struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}
impl ErrorResponse  {
    pub fn new(e: CustomError) -> Self {
        match e {
            CustomError::MaxValidationError => Self {
                code: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                error: "Unprocessable parameter".to_string(),
                message: format!("Invalid max parameter. Must be > {} and <= {}", 0, MAX_RECS),
            },

            CustomError::OrderError => Self {
                code: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                error: "Unprocessable parameter".to_string(),
                message: "Invalid order parameter. Must be ASC or DESC".to_string(),
            },
            CustomError::OffsetError => Self {
                code: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                error: "Unprocessable parameter".to_string(),
                message: "Offset parameter must be >= 0".to_string(),
            },
            CustomError::FoodSortError => Self {
                code: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                error: "Unprocessable parameter".to_string(),
                message: "Invalid order parameter. Must be 'description', 'id', 'fdcid' or 'upc'".to_string(),
            },
            CustomError::Unknown=> Self {
                code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                error: "Internal server error".to_string(),
                message: "Unknown Internal Error".to_string(),
            },
        }
    }
}
