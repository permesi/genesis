use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use tracing::{debug, error, instrument};
use ulid::Ulid;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct Token {
    token: String,
}

#[utoipa::path(
    post,
    path= "/verify",
    responses (
        (status = 202, description = "Return token", body = [Token], content_type = "application/json"),
        (status = 403, description = "Token expired or invalid"),
    ),
    tag = "verify",
)]
#[instrument]
pub async fn verify(Extension(pool): Extension<PgPool>, payload: Json<Token>) -> impl IntoResponse {
    let token = &payload.token;

    match Ulid::from_string(token) {
        Ok(_) => (),
        Err(e) => {
            error!("Error while parsing token: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    }

    let query = "SELECT EXISTS(SELECT 1 FROM tokens WHERE id = $1::ulid AND id::timestamp > NOW() - INTERVAL '30 MINUTES') AS valid";

    match sqlx::query(query).bind(token).fetch_one(&pool).await {
        Ok(row) => {
            let valid: bool = row.get("valid");
            if valid {
                debug!("Token is valid");
                StatusCode::ACCEPTED
            } else {
                error!("Token is invalid");
                StatusCode::FORBIDDEN
            }
        }

        Err(e) => {
            error!("Error while verifying token: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
