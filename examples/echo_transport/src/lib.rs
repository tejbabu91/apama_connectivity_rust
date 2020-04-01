use rust_ap_connectivity::*;
use std::collections::HashMap;

pub struct EchoTransport {
    data: i64,
    hostside: HostSide
}

impl Transport for EchoTransport {
    fn start(&self) {
        println!("EchoTransport started with {}", self.data);
    }
    fn shutdown(&self) {
        println!("EchoTransport shutdown done");
    }
    fn hostReady(&self) {
        println!("EchoTransport handled hostReady");
    }
    fn deliverMessageTowardsTransport(&self, mut msg: Message) {
        println!("EchoTransport received message from host: {:?}", msg);
        // echo message back towards host
        let mut m = HashMap::new();
        m.insert(Data::String("str".to_string()), Data::String("Hello from Rust!".to_string()));
        m.insert(Data::String("name".to_string()), Data::String("value".to_string()));
        m.insert(Data::Integer(35), Data::List(vec![Data::String(format!("Sending back {}", msg.payload)), Data::Boolean(true)]));

        // remove the channel before echoing it back since we may want a different channel in the opposite direction
        msg.metadata.remove(&Data::String("sag.channel".to_string()));
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
    fn new(h: HostSide, config: HashMap<Data,Data>) -> Box<dyn Transport> {
        println!("Creating transport with config {:?}", config);
        Box::new(EchoTransport{data: 43, hostside: h})
    }
}
DECLARE_CONNECTIVITY_TRANSPORT!(EchoTransport);
