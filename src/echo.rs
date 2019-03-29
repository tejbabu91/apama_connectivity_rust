
#[macro_use]
use crate::api::*;
use std::collections::HashMap;

pub struct MyTransport {
    data: i64,
    hostSide: HostSide
}

impl Transport for MyTransport {
    fn start(&self) {
        println!("MyTransport started with {}", self.data);
    }
    fn shutdown(&self) {
        println!("MyTransport shutdown done");
    }
    fn hostReady(&self) {
        println!("MyTransport handled hostReady");
    }
    fn deliverMessageTowardsTransport(&self, msg: Message) {
        println!("MyTransport received message from host: {:?}", msg);
        let msg = Message {
            payload: Data::Integer(123),
            metadata: HashMap::new()
        };
        self.getHostSide().sendMessageTwoardsHost(msg);
        // Send some more messages back to host for testing
        let msg = Message {
            payload: Data::String("Hello from transport".to_string()),
            metadata: HashMap::new()
        };
        self.getHostSide().sendMessageTwoardsHost(msg);

        let msg = Message {
            payload: Data::Float(123.45),
            metadata: HashMap::new()
        };
        self.getHostSide().sendMessageTwoardsHost(msg);

        let msg = Message {
            payload: Data::Boolean(true),
            metadata: HashMap::new()
        };
        self.getHostSide().sendMessageTwoardsHost(msg);

        let msg = Message {
            payload: Data::None,
            metadata: HashMap::new()
        };
        self.getHostSide().sendMessageTwoardsHost(msg);
    }
    fn getHostSide(&self) -> HostSide {
        self.hostSide
    }
}

impl MyTransport {
    pub fn new(h: HostSide, config: HashMap<Data,Data>) -> Box<Transport> {
        println!("Creating transport with config {:?}", config);
        Box::new(MyTransport{data: 43, hostSide: h})
    }
}

// DefineTrasport!(MyTransport);