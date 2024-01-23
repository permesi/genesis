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
pub struct Root {
    token: String,
}

#[utoipa::path(
    get,
    path= "/",
    responses (
        (status = 200, description = "Database connection is healthy", body = [Health]),
        (status = 503, description = "Database connection is unhealthy", body = [Health])
    )
)]
// axum handler for root path /
#[instrument]
pub async fn root(method: Method, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    debug!(method = ?method, "HTTP request method: {}", method);

    let token = Ulid::new();

    let query = "INSERT INTO tokens (token) VALUES ($1)";
    sqlx::query(query)
        .bind(token.to_string())
        .execute(&pool)
        .await
        .map_err(|err| {
            error!("Failed to insert token into database: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
        })
        .map(|_| {
            let token = Root {
                token: token.to_string(),
            };

            debug!("Token: {} inserted into database", token.token);
            (StatusCode::OK, Json(&token).into_response())
        })
}
