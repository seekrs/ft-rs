use std::str::FromStr;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub enum ErrorType {
  #[serde(rename = "invalid_client")]
  InvalidClient,
  #[serde(rename = "invalid_grant")]
  InvalidGrant,
  #[serde(rename = "invalid_request")]
  InvalidRequest,
  #[serde(rename = "invalid_scope")]
  InvalidScope,
  #[serde(rename = "unauthorized_client")]
  UnauthorizedClient,
  #[serde(rename = "unsupported_grant_type")]
  UnsupportedGrantType,
  #[serde(rename = "unknown")]
  Unknown
}

impl FromStr for ErrorType {
  type Err = FtError;

  fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
    Ok(serde_json::from_str::<ErrorType>(format!("\"{}\"", s).as_str()).unwrap_or(ErrorType::Unknown))
  }
}

#[derive(Debug, Error)]
pub enum FtError {
  #[error("Couldn't build the reqwest client")]
  ReqwestBuilderError(reqwest::Error),
  #[error("Error while sending request")]
  ReqwestError(#[from] reqwest::Error),
  #[error("Error while deserializing API response")]
  SerdeError {
    #[from]
    source: serde_json::Error
  },
  #[error("API error: {error:?}: {error_description:?}")]
  ApiError {
    error: ErrorType,
    error_description: String
  },
}

impl FtError {
  pub fn from_api_error(error: ErrorType, error_description: String) -> Self {
    Self::ApiError {
      error,
      error_description
    }
  }
}

pub type Result<T> = std::result::Result<T, FtError>;