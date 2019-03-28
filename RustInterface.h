#ifndef _RUST_CPP_INTERFACE_H_
#define _RUST_CPP_INTERFACE_H_
#include "RustTransport.h"
struct Data {
	long a;
	long b;
};

extern "C" {
	int add(int first, int second);
	void* rust_create_transport();
	void call_back_from_c(void*);
	void send_data_towards_transport(Data*);
	void send_msg_towards_transport(sag_underlying_message_t*);
}



#endif