use crate::{
    cli::globals::GlobalArgs,
    genesis::handlers::{
        headers::__path_headers, health, health::__path_health, token, token::__path_token,
    },
    vault,
};
use anyhow::{Context, Result};
use axum::{
    http::{HeaderName, HeaderValue, Method},
    response::Redirect,
    routing::{get, post},
    Extension, Router,
};
use mac_address::get_mac_address;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::{net::TcpListener, sync::mpsc};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    propagate_header::PropagateHeaderLayer,
    set_header::SetRequestHeaderLayer,
    trace::TraceLayer,
};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod handlers;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub const GIT_COMMIT_HASH: &str = if let Some(hash) = built_info::GIT_COMMIT_HASH {
    hash
} else {
    ":-("
};

#[derive(OpenApi)]
#[openapi(
    paths(health, headers, token),
    components(
        schemas(health::Health, token::Token)
    ),
    tags(
        (name = "genesis", description = "Token Zero generator API"),
    )

)]
struct ApiDoc;

pub async fn new(port: u16, dsn: String, globals: &GlobalArgs) -> Result<()> {
    // Renew vault token, gracefully shutdown if failed
    let (tx, mut rx) = mpsc::unbounded_channel();

    vault::renew::try_renew(globals, tx).await?;

    // Connect to database
    let pool = PgPoolOptions::new()
        .min_connections(1)
        .max_connections(5)
        .max_lifetime(Duration::from_secs(60 * 2))
        .test_before_acquire(true)
        .connect(&dsn)
        .await
        .context("Failed to connect to database")?;

    let swagger = SwaggerUi::new("/ui/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi());

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(|| async { Redirect::to("https://permesi.dev") }))
        .route("/headers", get(handlers::headers))
        .route("/health", get(handlers::health).options(handlers::health))
        .route("/token", get(handlers::token))
        .route("/verify", post(handlers::verify))
        .merge(swagger)
        .layer(
            ServiceBuilder::new()
                .layer(Extension(pool))
                .layer(PropagateHeaderLayer::new(HeaderName::from_static(
                    "x-request-id",
                )))
                .layer(SetRequestHeaderLayer::if_not_present(
                    HeaderName::from_static("x-request-id"),
                    |_req: &_| {
                        let node_id: [u8; 6] = node_id();
                        let uuid = uuid::Uuid::now_v1(&node_id);
                        HeaderValue::from_str(uuid.to_string().as_str()).ok()
                    },
                ))
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        );

    let listener = TcpListener::bind(format!("::0:{port}")).await?;

    info!("Listening on [::]:{}", port);

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async move {
            rx.recv().await;
            info!("Gracefully shutdown");
        })
        .await?;

    Ok(())
}

#[must_use]
pub fn node_id() -> [u8; 6] {
    get_mac_address().map_or([0; 6], |mac| {
        mac.map_or([0; 6], |mac| {
            let bytes = mac.bytes();
            let mut node_id = [0; 6];
            node_id.copy_from_slice(&bytes);
            node_id
        })
    })
}
