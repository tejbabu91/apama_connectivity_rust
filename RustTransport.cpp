/*
 * $Copyright (c) 2019 Software AG, Darmstadt, Germany and/or Software AG USA Inc., Reston, VA, USA, and/or its subsidiaries and/or its affiliates and/or their licensors.$
 * Use, reproduction, transfer, publication or disclosure is prohibited except as specifically provided for in your License Agreement with Software AG
 * $Revision: 321386 $ $Date: 2017-12-14 10:26:04 +0000 (Thu, 14 Dec 2017) $
 */

#include <sag_connectivity_plugins.hpp>
// #include <sag_connectivity_threading.h>
#include "RustTransport.h"
#include <vector>
#include<sstream>
#include<iostream>
#include <cmath>
#include "RustInterface.h"

using namespace com::softwareag::connectivity;

namespace apamax {
namespace rust {
	RustTransport::RustTransport(const TransportConstructorParameters &params)
		: AbstractSimpleTransport(params)
	{		
		auto &conf = this->config;
		rustTransport = rust_transport_create(this, 
			reinterpret_cast<sag_underlying_map_t*>(&conf));
	}

	RustTransport::~RustTransport() {
		rust_transport_destroy(rustTransport);
	}

	void RustTransport::start() {
		rust_transport_start(rustTransport);

		// Message msg;
		// {
		// 	map_t m;
		// 	m.insert(data_t("k_str"), data_t("v_str"));
		// 	m.insert(data_t("k_num"), data_t(2.34));
		// 	m.insert(data_t("k_bool"), data_t(true));
		// 	list_t l;
		// 	l.push_back(data_t("str"));
		// 	l.push_back(data_t(42.0));
		// 	l.push_back(data_t(true));
		// 	l.push_back(data_t(l.copy()));
		// 	l.push_back(data_t(m.copy()));
		// 	//m.insert(data_t("k_list"), data_t(std::move(l)));
		// 	//m.insert(data_t("k_map"), m.copy());
		// 	msg.setPayload(data_t(std::move(l)));
		// }
		//msg.setPayload(data_t("some string"));
		//logger.info("Sending msg: %s", to_string(msg).c_str());
		// deliverMessageTowardsTransport(msg);
	}

	/** Stop the plugin and wait for the request-handling thread */
	void RustTransport::shutdown()
	{
		rust_transport_shutdown(rustTransport);
	}

	/** Parse the request and queue it for later servicing */
	void RustTransport::deliverMessageTowardsTransport(Message &m)
	{
		//logger.info("C++ deliverMessageTowardsTransport: %s", to_string(m).c_str());
		rust_send_msg_towards_transport(rustTransport, reinterpret_cast<sag_underlying_message_t*>(&m));
	}

	void RustTransport::towardsHost(Message *m) {
		logger.info("C++ sending msg toward host: %s", to_string(*m).c_str());
		hostSide->sendBatchTowardsHost(m, m+1);
	}

	void RustTransport::hostReady() {
		rust_transport_hostReady(rustTransport);
	}

	

}} // apamax.rust


void rust_send_msg_towards_host(void* ptr, sag_underlying_message_t* msg) {
	apamax::rust::RustTransport* t = reinterpret_cast<apamax::rust::RustTransport*>(ptr);
	t->towardsHost(reinterpret_cast<Message*>(msg));
}


sag_underlying_data_t* create_cpp_data_t_empty() {
	return reinterpret_cast<sag_underlying_data_t*>(new data_t());
}

sag_underlying_data_t* create_cpp_data_t_bool(bool val) {
	return reinterpret_cast<sag_underlying_data_t*>(new data_t(val));
}

sag_underlying_data_t* create_cpp_data_t_int64(int64_t val) {
	return reinterpret_cast<sag_underlying_data_t*>(new data_t(val));
}

sag_underlying_data_t* create_cpp_data_t_double(double val) {
	return reinterpret_cast<sag_underlying_data_t*>(new data_t(val));
}

sag_underlying_vector_t* create_cpp_list_t_with_capacity(int64_t capacity) {
	return reinterpret_cast<sag_underlying_vector_t*>(new list_t(capacity));
}

void append_to_list_t(sag_underlying_vector_t *l, sag_underlying_data_t *d) {
	list_t *l_class = reinterpret_cast<list_t *>(l);
	data_t *d_class = reinterpret_cast<data_t *>(d);
	l_class->push_back(std::move(*d_class));
	delete d_class;
}

sag_underlying_data_t* create_cpp_data_t_list_t(sag_underlying_vector_t *val) {
	list_t *l_class = reinterpret_cast<list_t *>(val);
	return reinterpret_cast<sag_underlying_data_t*>(new data_t(std::move(*l_class)));
}

sag_underlying_map_t* create_cpp_map_t() {
	return reinterpret_cast<sag_underlying_map_t*>(new map_t());
}

void insert_into_map_t(sag_underlying_map_t *m, sag_underlying_data_t *key, sag_underlying_data_t *value) {
	map_t *m_class = reinterpret_cast<map_t *>(m);
	data_t *key_class = reinterpret_cast<data_t *>(key);
	data_t *value_class = reinterpret_cast<data_t *>(value);
	m_class->insert(std::move(*key_class), std::move(*value_class));
	delete key_class;
	delete value_class;
}

sag_underlying_data_t* create_cpp_data_t_map_t(sag_underlying_map_t *val) {
	map_t *m_class = reinterpret_cast<map_t *>(val);
	return reinterpret_cast<sag_underlying_data_t*>(new data_t(std::move(*m_class)));
}