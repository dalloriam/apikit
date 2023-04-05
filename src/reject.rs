//! Commonly used rejections and recovery procedures.
use std::fmt::Display;

use axum::http::StatusCode;

use axum::response::{IntoResponse, Response};

use miette::Diagnostic;
use serde::Serialize;

use crate::reply;

const MESSAGE_NOT_FOUND: &str = "not found";
const MESSAGE_FORBIDDEN: &str = "forbidden";
const MESSAGE_INTERNAL_SERVER_ERROR: &str = "internal server error";

#[derive(Debug, Diagnostic, Serialize)]
#[serde(untagged)]
pub enum HTTPError {
    BadRequest {
        error: String,
    },
    Forbidden,
    NotFound,
    InternalServerError {
        error: String,
        backtrace: Option<String>,
    },
}

impl Display for HTTPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadRequest { error } => write!(f, "bad request: {}", error),
            Self::Forbidden => write!(f, "forbidden"),
            Self::NotFound => write!(f, "not found"),
            Self::InternalServerError {
                error,
                backtrace: Some(backtrace),
            } => write!(f, "internal server error: {}\n{}", error, backtrace),
            Self::InternalServerError {
                error,
                backtrace: None,
            } => write!(f, "internal server error: {}", error),
        }
    }
}

impl std::error::Error for HTTPError {}

impl HTTPError {
    pub fn bad_request<S: ToString>(s: S) -> Self {
        Self::BadRequest {
            error: s.to_string(),
        }
    }

    pub fn internal_server_error<E: ToString>(e: E) -> Self {
        Self::InternalServerError {
            error: e.to_string(),
            backtrace: None, // TODO: Properly capture backtrace
        }
    }
}

impl IntoResponse for HTTPError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest { error } => reply::error(error, StatusCode::BAD_REQUEST),
            Self::Forbidden => reply::error(MESSAGE_FORBIDDEN, StatusCode::FORBIDDEN),
            Self::NotFound => reply::error(MESSAGE_NOT_FOUND, StatusCode::NOT_FOUND),
            Self::InternalServerError {
                ref error,
                ref backtrace,
            } => {
                if let Some(backtrace) = backtrace {
                    tracing::error!("{error}\n{backtrace}");
                } else {
                    tracing::error!("{error}");
                }
                reply::error(
                    MESSAGE_INTERNAL_SERVER_ERROR,
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        }
    }
}
