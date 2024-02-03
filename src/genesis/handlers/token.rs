use axum::{
    extract::{Extension, Query},
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
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct Token {
    token: String,
}

#[derive(IntoParams, Debug, Deserialize, Default)]
#[into_params(parameter_in = Query)]
pub struct ClientArgs {
    // uuid of the client
    client_id: String,
}

#[utoipa::path(
    get,
    path= "/token",
    params(ClientArgs),
    responses (
        (status = 200, description = "Return token", body = [Token]),
        (status = 500, description = "Error creating the token")
    ),
    tag = "token",
)]
#[instrument]
pub async fn token(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    query: Option<Query<ClientArgs>>,
) -> impl IntoResponse {
    let args = if let Some(args) = query {
        Query(args)
    } else {
        error!("Failed to get query parameters");
        return Err((StatusCode::BAD_REQUEST, "Missing Client ID".to_string()));
    };

    let client_uuid = match args.client_id.parse::<Uuid>() {
        Ok(uuid) => uuid,
        Err(err) => {
            error!("Failed to parse uuid: {}", err);
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid Client ID format".to_string(),
            ));
        }
    };

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

    debug!("Client ID: {}", client_id);

    // start transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            error!("Failed to start transaction: {}", err);

            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to start transaction".to_string(),
            ));
        }
    };

    let query = "INSERT INTO tokens (id, client_id) VALUES ($1::ulid, $2) RETURNING id::text";
    let result = match sqlx::query(query)
        .bind(token.to_string())
        .bind(client_id)
        .fetch_one(&mut *tx)
        .await
    {
        Ok(row) => {
            let token_id: String = row.get("id");

            let metadata_query =
                "INSERT INTO metadata (id, ip_address, country, user_agent) VALUES ($1::ulid, $2, $3, $4)";
            sqlx::query(metadata_query)
                .bind(token_id)
                .bind(ip_address)
                .bind(country)
                .bind(ua)
                .execute(&mut *tx)
                .await
        }

        Err(err) => Err(err),
    };

    match result {
        Ok(_) => match tx.commit().await {
            Ok(()) => Ok((
                StatusCode::OK,
                Json(Token {
                    token: token.to_string(),
                }),
            )),

            Err(err) => {
                error!("Failed to commit transaction: {}", err);

                Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
            }
        },

        Err(err) => {
            match tx.rollback().await {
                Ok(()) => debug!("Rolled back transaction"),

                Err(err) => error!("Failed to rollback transaction: {}", err),
            };

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
