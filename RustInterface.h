#ifndef _RUST_CPP_INTERFACE_H_
#define _RUST_CPP_INTERFACE_H_
#include "RustTransport.h"
struct Data {
	long a;
	long b;
};

extern "C" {
	int add(int first, int second);
	void call_back_from_c(void*);
	void send_data_towards_transport(Data*);
	void* rust_transport_create();
	void rust_transport_send_msg_towards(void*, sag_underlying_message_t*);
	void rust_transport_start(void*);
	void rust_transport_shutdown(void*);
	void rust_transport_hostReady(void*);
}



#endif