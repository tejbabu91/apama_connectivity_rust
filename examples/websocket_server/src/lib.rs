use futures_util::{SinkExt, StreamExt};
use log::LevelFilter;
use log::{error, info};
use rust_ap_connectivity::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::protocol::Message as WSMessage;
use tungstenite::Result as TResult;
pub struct WebSocketServerConfig {
    host: String,
    port: String,
    config: HashMap<String, Data>,
}

type AMConnections = Arc<Mutex<HashMap<u64, Sender<WSMessage>>>>;
type AMIDTracker = Arc<AtomicUsize>;

pub struct WebSocketTransport {
    config: WebSocketServerConfig,
    hostside: HostSide,
    transport_params: TransportConstructorParameters,
    runtime: Option<tokio::runtime::Runtime>,
    connections: AMConnections,
    id_tracker: AMIDTracker,
}

fn websocket_message_to_message(m: WSMessage, id: usize) -> Message {
    let mut m = Message {
        payload: Data::String(format!("{}", m)),
        metadata: HashMap::new(),
    };
    m.metadata
        .insert(Data::String("id".to_string()), Data::Integer(id as i64));
    m
}

async fn handle_connection(
    peer: SocketAddr,
    ws_stream: WebSocketStream<TcpStream>,
    id: usize,
    conn_arc: AMConnections,
    mut to_host_channel: Sender<Message>,
) -> TResult<()> {
    let (mut sender, mut receiver) = ws_stream.split();

    info!("New WebSocket connection: {}", peer);
    let (tx, mut rx): (Sender<WSMessage>, Receiver<WSMessage>) = channel(100);

    conn_arc.lock().await.insert(id as u64, tx);

    tokio::spawn(async move {
        tokio::spawn(async move {
            while let Some(m) = rx.recv().await {
                if let Err(e) = sender.send(m).await {
                    rx.close();
                    error!("client connection closed: {}", e);
                    break;
                }
            }
        });

        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                if msg.is_text() || msg.is_binary() {
                    to_host_channel
                        .send(websocket_message_to_message(msg, id))
                        .await
                        .expect("send into buffer");
                }
            }
        }
    });

    Ok(())
}

impl Transport for WebSocketTransport {
    fn start(&mut self) {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();

        let host = self.config.host.clone();
        let port = self.config.port.clone();

        let conn_arc = Arc::clone(&self.connections);
        let id_arc = Arc::clone(&self.id_tracker);

        let (tx, mut rx): (Sender<Message>, Receiver<Message>) = channel(100);

        let host_side = self.hostside;

        self.runtime.as_ref().unwrap().spawn(async move {
            while let Some(m) = rx.next().await {
                host_side.sendMessageTowardsHost(m);
            }
        });

        self.runtime.as_ref().unwrap().spawn(async move {
            log::set_max_level(LevelFilter::Debug);
            let addr = format!("{}:{}", host, port);
            info!("Listening on: {}", addr);
            let mut listener = TcpListener::bind(&addr).await.expect("Can't listen");

            while let Ok((stream, _)) = listener.accept().await {
                let peer = stream
                    .peer_addr()
                    .expect("connected streams should have a peer address");
                info!("Peer address: {}", peer);

                let conn_arc = Arc::clone(&conn_arc);
                let id_arc = Arc::clone(&id_arc);

                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        let id = id_arc.fetch_add(1, Ordering::SeqCst);
                        tokio::spawn(handle_connection(peer, ws_stream, id, conn_arc, tx.clone()));
                    }
                    Err(e) => match e {
                        Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                        err => error!("Error processing connection: {}", err),
                    },
                }
            }
        });
        info!("WebSocketTransport started");
    }
    fn shutdown(&mut self) {
        // dropping runtime reference kills all tasks
        self.runtime = None;
        info!("WebSocketTransport shutdown done");
    }
    fn hostReady(&mut self) {
        info!("WebSocketTransport handled hostReady");
    }
    fn deliverMessageTowardsTransport(&mut self, msg: Message) {
        // info!("WebSocketTransport received message from host: {:?}", msg);

        let wsm = WSMessage::from(format!(
            "{}",
            msg.payload
                .as_opt_string()
                .or(Some(&"no_string".to_string()))
                .unwrap()
        ));
        let id = match msg.metadata.get(&Data::String(String::from("id"))) {
            Some(Data::Integer(v)) => *v,
            _ => return,
        };
        let local_arc = Arc::clone(&self.connections);
        self.runtime.as_ref().unwrap().spawn(async move {
            if let Some(tx) = local_arc.lock().await.get_mut(&(id as u64)) {
                let mut tmptx = tx.clone();
                tmptx.send(wsm).await.expect("sending to client channel");
            }
        });
    }
    fn getHostSide(&mut self) -> &mut HostSide {
        &mut self.hostside
    }
    fn getParams(&mut self) -> &mut TransportConstructorParameters {
        &mut self.transport_params
    }
    fn new(h: HostSide, params: TransportConstructorParameters) -> Box<dyn Transport> {
        info!("Creating transport with config {:?}", params);

        // copy all string keys into cfg
        let cfg: HashMap<String, Data> = params
            .getConfig()
            .iter()
            .filter(|(k, _)| k.is_string())
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        let host = cfg
            .get(&String::from("host"))
            .or(Some(&Data::String("127.0.0.1".to_string())))
            .unwrap()
            .to_string();
        let port = cfg
            .get(&String::from("port"))
            .or(Some(&Data::String("3999".to_string())))
            .unwrap()
            .to_string();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        Box::new(WebSocketTransport {
            config: WebSocketServerConfig {
                host,
                port,
                config: cfg,
            },
            hostside: h,
            transport_params: params,
            runtime: Some(runtime),
            connections: Arc::new(Mutex::from(HashMap::new())),
            id_tracker: Arc::new(AtomicUsize::new(1)),
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
