use axum::{
    extract::Extension,
    http::{Method, StatusCode},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, error, instrument};
use ulid::Ulid;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct Token {
    token: String,
}

#[utoipa::path(
    get,
    path= "/",
    responses (
        (status = 200, description = "Return token", body = [Token]),
        (status = 500, description = "Error creating the token", body = [Token])
    )
)]
#[instrument]
pub async fn root(method: Method, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let token = Ulid::new();

    let query = "INSERT INTO tokens (token) VALUES (?)";
    sqlx::query(query)
        .bind(token.to_string())
        .execute(&pool)
        .await
        .map_err(|err| {
            error!("Failed to insert token into database: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
        })
        .map(|_| {
            let token = Token {
                token: token.to_string(),
            };

            debug!("Token: {} inserted into database", token.token);
            (StatusCode::OK, Json(&token).into_response())
        })
}
