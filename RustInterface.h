#ifndef _RUST_CPP_INTERFACE_H_
#define _RUST_CPP_INTERFACE_H_
#include "RustTransport.h"
struct Data {
	long a;
	long b;
};

extern "C" {
	void* rust_transport_create(void*);
	void rust_send_msg_towards_transport(void*, sag_underlying_message_t*);
	void rust_transport_start(void*);
	void rust_transport_shutdown(void*);
	void rust_transport_hostReady(void*);
	void rust_send_msg_towards_host(void*, sag_underlying_message_t*);
}



#endif