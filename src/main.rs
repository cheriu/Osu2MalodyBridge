use std::sync::Arc;

use tracing::info;
use tracing_subscriber::EnvFilter;

use osu2malody_store::config::Config;
use osu2malody_store::router::create_router;
use osu2malody_store::services::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // rustls 0.23 requires an explicit crypto provider when multiple backends
    // are available (both aws-lc-rs and ring are pulled in by dependencies).
    let _ = rustls::crypto::ring::default_provider().install_default();

    let config = Config::load()?;
    info!("Config loaded");

    let osu = rosu_v2::Osu::new(
        config.malody.osu.client_id.expect("validated"),
        config.malody.osu.client_secret.clone().expect("validated"),
    )
    .await?;
    info!("osu! API v2 client initialized");

    let state = Arc::new(AppState::new(config.clone(), osu));
    let app = create_router(state);

    // Raw bind address for socket: bracket IPv6 (e.g. "[::]:8080"), no bracket for IPv4
    let bind_addr = if config.server.bind_address.contains(':') {
        format!("[{}]:{}", config.server.bind_address, config.server.port)
    } else {
        format!("{}:{}", config.server.bind_address, config.server.port)
    };

    match (&config.server.tls_cert, &config.server.tls_key) {
        (Some(cert), Some(key)) => {
            info!("Starting HTTPS server on {}", bind_addr);
            let tls_config =
                axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key).await?;
            axum_server::bind_rustls(bind_addr.parse()?, tls_config)
                .serve(app.into_make_service())
                .await?;
        }
        _ => {
            info!("Starting HTTP server on {}", bind_addr);
            let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
