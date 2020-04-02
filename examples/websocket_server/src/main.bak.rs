
use tokio::net::TcpListener;
use websocket_server::websocket::accept_connection;
use log::*;
use futures::join;
use tokio::runtime::Runtime;

async fn main2() {
    env_logger::builder().filter_level(LevelFilter::Debug).init();
    log::set_max_level(LevelFilter::Debug);

    let addr = "127.0.0.1:9002";
    let mut listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream));
    }
}

fn main() {

    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.spawn(async move {
        main2().await;
    });
}