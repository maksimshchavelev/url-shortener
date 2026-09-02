use axum::routing::{any, post};
use axum::{Router, extract::DefaultBodyLimit, middleware};
use std::net::SocketAddr;
use std::{env, process, sync::Arc};
use tokio::{net, signal};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use url_shortener::{application, domain, http, infra};

#[tokio::main]
async fn main() -> Result<(), domain::Error> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // ----------- env -----------
    let database_url = env::var("SHORTENER_DATABASE_URL").unwrap_or_else(|e| {
        error!("Failed to read SHORTENER_DATABASE_URL env variable: {e}");
        process::exit(1)
    });

    let database_connections = env::var("SHORTENER_DATABASE_CONNECTIONS")
        .unwrap_or("64".to_string())
        .parse::<u32>()
        .unwrap_or_else(|e| {
            error!("Failed to parse SHORTENER_DATABASE_CONNECTIONS env variable: {e}");
            process::exit(1)
        });

    let server_port = env::var("SHORTENER_SERVER_PORT").unwrap_or_else(|e| {
        error!("Failed to read SHORTENER_SERVER_PORT env variable: {e}");
        process::exit(1)
    });

    let server_listen_ip = env::var("SHORTENER_LISTEN_IP").unwrap_or_else(|e| {
        error!("Failed to read SHORTENER_LISTEN_IP env variable: {e}");
        process::exit(1)
    });

    // ----------- infra -----------
    let repository = infra::PostgresRepository::new(&database_url, database_connections).await?;

    info!("Run database migrations...");
    repository.migrate().await?;
    info!("Migrations applied");

    let generator = application::CodeGenerator::new();
    let link_service = application::LinkService::new(Box::new(generator), Box::new(repository));
    let state = domain::AppState::new(Box::new(link_service));

    // ----------- routes -----------
    let app = Router::new()
        .route("/{code}", any(http::Handlers::handle_redirect))
        .route("/create", post(http::Handlers::handle_create))
        .with_state(Arc::new(state))
        .layer(middleware::from_fn(http::Middlewares::ip_logger))
        .layer(DefaultBodyLimit::max(1024 * 8));

    // ----------- server -----------
    let listener = net::TcpListener::bind(format!("{}:{}", server_listen_ip, server_port))
        .await
        .map_err(domain::Error::from_internal)?;

    info!(
        "Listening {server_listen_ip}:{server_port} with {database_connections} database connections"
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(domain::Error::from_internal)?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
