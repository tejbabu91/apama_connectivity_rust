use rust_ap_connectivity::*;
use std::collections::HashMap;

pub struct EchoTransport {
    hostside: HostSide,
    params: TransportConstructorParameters,
}

impl Transport for EchoTransport {
    fn start(&mut self) {
        println!("EchoTransport started with {:?}", self.params.getConfig());
    }
    fn shutdown(&mut self) {
        println!("EchoTransport shutdown done");
    }
    fn hostReady(&mut self) {
        println!("EchoTransport handled hostReady");
    }
    fn deliverMessageTowardsTransport(&mut self, mut msg: Message) {
        println!("EchoTransport received message from host: {:?}", msg);
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

        // remove the channel before echoing it back since we may want a different channel in the opposite direction
        msg.metadata
            .remove(&Data::String("sag.channel".to_string()));
        let m = Message {
            // payload: Data::String(format!("Sending back {}", msg.payload)),
            payload: Data::Map(m),
            metadata: msg.metadata,
        };
        self.getHostSide().sendMessageTowardsHost(m);
    }
    fn getHostSide(&mut self) -> &mut HostSide {
        &mut self.hostside
    }
    fn getParams(&mut self) -> &mut TransportConstructorParameters {
        &mut self.params
    }
    fn new(hostside: HostSide, params: TransportConstructorParameters) -> Box<dyn Transport> {
        println!("Creating transport with config: {:?}", params.getConfig());
        Box::new(EchoTransport { hostside, params })
    }
}

impl std::ops::Drop for EchoTransport {
    fn drop(&mut self) {
        println!("EchoTransport Dropped");
    }
}
DECLARE_CONNECTIVITY_TRANSPORT!(EchoTransport);
