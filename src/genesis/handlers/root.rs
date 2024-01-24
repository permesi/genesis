use axum::{
    extract::Extension,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::env;
use std::net::IpAddr;
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
pub async fn root(Extension(pool): Extension<PgPool>, headers: HeaderMap) -> impl IntoResponse {
    let token = Ulid::new();

    // Get the IP address from the headers using the environment variable if it exists
    // otherwise use default to CF-Connecting-IP
    let ip_address = ip_from_headers(
        env::var("GENESIS_COUNTRY_HEADER").unwrap_or_else(|_| "CF-Connecting-IP".to_string()),
        &headers,
    );

    // Get the country from the headers using the environment variable if it exists
    let country = headers
        .get(env::var("GENESIS_COUNTRY_HEADER").unwrap_or_else(|_| "CF-IPCountry".to_string()))
        .map(|country| country.to_str().ok());

    // User-Agent is optional
    let ua = headers.get("User-Agent").and_then(|v| v.to_str().ok());

    let query = "INSERT INTO tokens (token) VALUES ($1) RETURNING id";
    let tx = sqlx::query(query)
        .bind(token.to_string())
        .fetch_optional(&pool)
        .await;

    match tx {
        Ok(Some(row)) => {
            let token_id: i64 = row.get("id");

            let metadata_query =
                "INSERT INTO metadata (id, ip_address, country, user_agent) VALUES ($1, $2, $3, $4)";

            sqlx::query(metadata_query)
                .bind(token_id)
                .bind(ip_address)
                .bind(country)
                .bind(ua)
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

        Ok(None) => {
            error!("Failed to insert token into database");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve id after inserting token".to_string(),
            ))
        }

        Err(err) => {
            error!("Failed to insert token into database: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        }
    }
}

fn ip_from_headers(header: String, headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get(header)
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.parse::<IpAddr>().ok())
}
