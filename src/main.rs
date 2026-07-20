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

    let osu = rosu_v2::Osu::builder()
        .client_id(config.malody.osu.client_id.expect("validated"))
        .client_secret(config.malody.osu.client_secret.clone().expect("validated"))
        .ratelimit(5)
        .build()
        .await?;
    info!("osu! API v2 client initialized");

    let state = Arc::new(AppState::new(config.clone(), osu));
    let app = create_router(state);

    let listen_addr = format!("[::]:{}", config.server.port);

    match (&config.server.tls_cert, &config.server.tls_key) {
        (Some(cert), Some(key)) => {
            info!("Starting HTTPS server on {}", listen_addr);
            let tls_config =
                axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key).await?;
            axum_server::bind_rustls(listen_addr.parse()?, tls_config)
                .serve(app.into_make_service())
                .await?;
        }
        _ => {
            info!("Starting HTTP server on {}", listen_addr);
            let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
