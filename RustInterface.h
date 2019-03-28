#ifndef _RUST_CPP_INTERFACE_H_
#define _RUST_CPP_INTERFACE_H_

extern "C" {
	int add(int first, int second);
	void* create_transport();
	void call_back_from_c(void*);
}

#endif