use rust_ap_connectivity::*;

pub struct DiagCodec {
    host_side: HostSide,
    transport_side: TransportSide,
    params: CodecConstructorParameters,
    tag: String,
}

impl Codec for DiagCodec {
    fn start(&mut self) {
        println!("DiagnosticCodec[{}] Started", self.tag);
    }
    fn shutdown(&mut self) {
        println!("DiagnosticCodec[{}] shutdown done", self.tag);
    }
    fn hostReady(&mut self) {
        println!("DiagnosticCodec[{}] handled hostReady", self.tag);
    }
    fn deliverMessageTowardsTransport(&mut self, msg: Message) {
        println!(
            "DiagnosticCodec[{}] Towards Transport: {:?} / {}",
            self.tag, msg.metadata, msg.payload
        );
        self.transport_side.sendMessageTowardsTransport(msg);
    }
    fn deliverMessageTowardsHost(&mut self, msg: Message) {
        println!(
            "DiagnosticCodec[{}] Towards Host: {:?} / {}",
            self.tag, msg.metadata, msg.payload
        );
        self.host_side.sendMessageTowardsHost(msg);
    }
    fn getHostSide(&mut self) -> &mut HostSide {
        &mut self.host_side
    }
    fn getTransportSide(&mut self) -> &mut TransportSide {
        &mut self.transport_side
    }
    fn getParams(&mut self) -> &mut CodecConstructorParameters {
        &mut self.params
    }
    fn new(
        host_side: HostSide,
        transport_side: TransportSide,
        params: CodecConstructorParameters,
    ) -> Box<dyn Codec> {
        let tag: String;
        if let Some(v) = params.getConfig().get(&Data::String(String::from("tag"))) {
            if let Data::String(s) = v {
                tag = s.to_string();
            } else {
                panic!("Expected string tag");
            }
        } else {
            tag = String::from("<none>");
        }
        let result = Box::new(DiagCodec {
            host_side,
            transport_side,
            params,
            tag,
        });
        println!(
            "DiagnosticCodec[{}] Created: {:?}",
            result.tag,
            result.params.getConfig()
        );
        result
    }
}

impl std::ops::Drop for DiagCodec {
    fn drop(&mut self) {
        println!("DiagnosticCodec[{}] Dropped", self.tag);
    }
}
DECLARE_CONNECTIVITY_CODEC!(DiagCodec);
