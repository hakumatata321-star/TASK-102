use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Account locked until {0}")]
    AccountLocked(String),

    #[error("Approval required")]
    ApprovalRequired { request_id: Uuid },

    #[error("Internal error")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    approval_request_id: Option<Uuid>,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Unauthorized(msg) => HttpResponse::Unauthorized().json(ErrorBody {
                error: msg.clone(),
                code: "UNAUTHORIZED".into(),
                approval_request_id: None,
            }),
            AppError::Forbidden(msg) => HttpResponse::Forbidden().json(ErrorBody {
                error: msg.clone(),
                code: "FORBIDDEN".into(),
                approval_request_id: None,
            }),
            AppError::NotFound(msg) => HttpResponse::NotFound().json(ErrorBody {
                error: msg.clone(),
                code: "NOT_FOUND".into(),
                approval_request_id: None,
            }),
            AppError::Validation(msg) => HttpResponse::BadRequest().json(ErrorBody {
                error: msg.clone(),
                code: "VALIDATION_ERROR".into(),
                approval_request_id: None,
            }),
            AppError::Conflict(msg) => HttpResponse::Conflict().json(ErrorBody {
                error: msg.clone(),
                code: "CONFLICT".into(),
                approval_request_id: None,
            }),
            AppError::AccountLocked(until) => HttpResponse::TooManyRequests().json(ErrorBody {
                error: format!("Account locked until {}", until),
                code: "ACCOUNT_LOCKED".into(),
                approval_request_id: None,
            }),
            AppError::ApprovalRequired { request_id } => {
                HttpResponse::Accepted().json(ErrorBody {
                    error: "Action requires approval".into(),
                    code: "APPROVAL_REQUIRED".into(),
                    approval_request_id: Some(*request_id),
                })
            }
            AppError::Internal(msg) => {
                log::error!("Internal error: {}", msg);
                HttpResponse::InternalServerError().json(ErrorBody {
                    error: "Internal server error".into(),
                    code: "INTERNAL_ERROR".into(),
                    approval_request_id: None,
                })
            }
        }
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => AppError::NotFound("Resource not found".into()),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                info,
            ) => AppError::Conflict(
                info.message().to_string(),
            ),
            _ => AppError::Internal(e.to_string()),
        }
    }
}

/// Helper to convert pool errors to AppError.
pub fn pool_err(e: impl std::fmt::Display) -> AppError {
    AppError::Internal(format!("Pool error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    #[test]
    fn test_unauthorized_response() {
        let err = AppError::Unauthorized("test".into());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_forbidden_response() {
        let err = AppError::Forbidden("test".into());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_not_found_response() {
        let err = AppError::NotFound("test".into());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_validation_response() {
        let err = AppError::Validation("test".into());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_conflict_response() {
        let err = AppError::Conflict("test".into());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_account_locked_response() {
        let err = AppError::AccountLocked("2026-01-01T00:00:00Z".into());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_approval_required_response() {
        let err = AppError::ApprovalRequired { request_id: Uuid::nil() };
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    #[test]
    fn test_internal_response() {
        let err = AppError::Internal("oops".into());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
