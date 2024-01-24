use axum::{
    extract::Extension,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::env;
use std::net::IpAddr;
use tracing::{debug, error, instrument};
use ulid::Ulid;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct Token {
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct Client {
    uuid: String,
}

#[utoipa::path(
    post,
    path= "/",
    responses (
        (status = 200, description = "Return token", body = [Token]),
        (status = 500, description = "Error creating the token", body = [Token])
    )
)]
#[instrument]
pub async fn root(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(payload): Json<Client>,
) -> impl IntoResponse {
    let client_uuid = Uuid::parse_str(&payload.uuid).unwrap_or_else(|err| {
        error!("Failed to parse uuid: {}", err);
        Uuid::nil()
    });

    debug!("Client UUID: {}", client_uuid);

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

    // get client id from the payload
    let query = "SELECT id FROM clients WHERE uuid = $1";
    let client_id = match sqlx::query(query).bind(client_uuid).fetch_one(&pool).await {
        Ok(row) => row.get::<i32, _>("id"),
        Err(err) => {
            error!("Failed to retrieve client id from database: {}", err);
            0
        }
    };

    let query = "INSERT INTO tokens (token, client_id) VALUES ($1, $2) RETURNING id";
    let tx = sqlx::query(query)
        .bind(token.to_string())
        .bind(client_id)
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
