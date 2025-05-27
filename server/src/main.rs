use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;

use clap::Parser;

#[tokio::main]
async fn main() {
    let opt = server::Opt::parse();

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }

    // enable console logging
    tracing_subscriber::fmt::init();

    let app = server::setup_app(opt.static_dir).await;

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    log::info!("listening on http://{}", sock_addr);

    let listener = tokio::net::TcpListener::bind(sock_addr).await.unwrap();

    axum::serve(listener, app)
        .await
        .expect("Unable to start server");
}
