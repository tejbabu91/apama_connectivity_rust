
use rust_ap_connectivity::*;
use std::collections::HashMap;
use log::LevelFilter;
use tokio::net::TcpListener;
use log::{info, error};
use std::cell::RefCell;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::Result as TResult;
use tokio::task;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::{Receiver, Sender};
use tungstenite::protocol::Message as WSMessage;
use futures_util::{SinkExt, StreamExt};

pub struct WebSocketServerConfig {
    host: String,
    port: String,
    otherConfig: HashMap<String, Data>,
}

pub struct WebSocketTransport {
    config: WebSocketServerConfig,
    hostside: HostSide,
    transportParams: TransportConstructorParameters,
    runtime: RefCell<Option<tokio::runtime::Runtime>>,
    connections: RefCell<HashMap<u64, Sender<WSMessage>>,
    id_tracker: RefCell<u64>,
}

impl WebSocketTransport {
    async fn accept_connection(&self, peer: SocketAddr, stream: TcpStream) {
        if let Err(e) = self.handle_connection(peer, stream).await {
            match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => error!("Error processing connection: {}", err),
            }
        }
    }

    async fn handle_connection(&self, peer: SocketAddr, stream: TcpStream) -> TResult<()> {
        let ws_stream = accept_async(stream).await.expect("Failed to accept");
        let (mut sender, mut receiver) = ws_stream.split();

        info!("New WebSocket connection: {}", peer);

        task::spawn(async move {
            let (mut tx, mut rx): (Sender<WSMessage>, Receiver<WSMessage>) = channel(100);

            task::spawn(async move {
                let mut closed = false;
                while let Some(m) = rx.recv().await {
                    if closed {
                        break;
                    }
                    for _ in 1..=10 {
                        let c = m.clone();
                        if let Err(e) = sender.send(c).await {
                            error!("client connection closed: {}", e);
                            closed = true;
                            break;
                        }
                    }
                }
            });

            while let Some(msg) = receiver.next().await {
                if let Ok(msg) = msg {
                    if msg.is_text() || msg.is_binary() {
                        tx.send(msg).await.expect("sent into buffer");
                        // ws_stream.send(msg).await.expect("sending failed");
                    }
                }
            }
        });

        Ok(())
    }
}

impl Transport for WebSocketTransport {

    fn start(&self) {

        let host = self.config.host.clone();
        let port = self.config.port.clone();

        self.runtime.borrow_mut().as_ref().unwrap().spawn(async move {
            env_logger::builder().filter_level(LevelFilter::Debug).init();
            log::set_max_level(LevelFilter::Debug);
        
            let addr = format!("{}:{}", host, port);
            info!("Listening on: {}", addr);
            let mut listener = TcpListener::bind(&addr).await.expect("Can't listen");
        
            while let Ok((stream, _)) = listener.accept().await {
                let peer = stream
                    .peer_addr()
                    .expect("connected streams should have a peer address");
                info!("Peer address: {}", peer);
        
                tokio::spawn(self.accept_connection(peer, stream));
            }
        });
        println!("WebSocketTransport started");
    }
    fn shutdown(&self) {
        // dropping runtime reference kills all tasks
        *self.runtime.borrow_mut() = None;
        std::thread::sleep(std::time::Duration::from_millis(10000));
        println!("WebSocketTransport shutdown done");
    }
    fn hostReady(&self) {
        println!("WebSocketTransport handled hostReady");
    }
    fn deliverMessageTowardsTransport(&self, msg: Message) {
        println!("WebSocketTransport received message from host: {:?}", msg);
        // echo message back towards host
        let mut m = HashMap::new();
        m.insert(
            Data::String("str".to_string()),
            Data::String("Hello from Rust!".to_string()),
        );
        m.insert(
            Data::String("name".to_string()),
            Data::String("value".to_string()),
        );
        m.insert(
            Data::Integer(35),
            Data::List(vec![
                Data::String(format!("Sending back {}", msg.payload)),
                Data::Boolean(true),
            ]),
        );
        let m = Message {
            // payload: Data::String(format!("Sending back {}", msg.payload)),
            payload: Data::Map(m),
            metadata: msg.metadata,
        };
        self.getHostSide().sendMessageTowardsHost(m);
    }
    fn getHostSide(&self) -> &HostSide {
        &self.hostside
    }
    fn getParams(&self) -> &TransportConstructorParameters {
        &self.transportParams
    }
    fn new(h: HostSide, params: TransportConstructorParameters) -> Box<dyn Transport> {
        println!("Creating transport with config {:?}", params);

        // move all string keys into cfg
        let cfg: HashMap<String, Data> = params.getConfig()
            .iter()
            .filter(|(k, _)| matches!(k, Data::String(_)))
            .map(|(k, v)| (k.get_string().unwrap().clone(), v.clone()))
            .collect();
        let host = cfg
            .get(&String::from("host"))
            .or(Some(&Data::String("127.0.0.1".to_string())))
            .unwrap()
            .get_string()
            .expect("host value should be of type string").clone();
        let port = cfg
            .get(&String::from("port"))
            .or(Some(&Data::String("3999".to_string())))
            .unwrap()
            .get_string()
            .expect("host value should be of type string").clone();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        Box::new(WebSocketTransport {
            config: WebSocketServerConfig {
                host,
                port,
                otherConfig: cfg,
            },
            hostside: h,
            transportParams: params,
            runtime: RefCell::from(Some(runtime)),
        })
    }
}
DECLARE_CONNECTIVITY_TRANSPORT!(WebSocketTransport);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
