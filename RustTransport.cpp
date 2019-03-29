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

		Message msg;
		{
			map_t m;
			m.insert(data_t("k_str"), data_t("v_str"));
			m.insert(data_t("k_num"), data_t(2.34));
			m.insert(data_t("k_bool"), data_t(true));
			list_t l;
			l.push_back(data_t("str"));
			l.push_back(data_t(42.0));
			l.push_back(data_t(true));
			l.push_back(data_t(l.copy()));
			l.push_back(data_t(m.copy()));
			//m.insert(data_t("k_list"), data_t(std::move(l)));
			//m.insert(data_t("k_map"), m.copy());
			msg.setPayload(data_t(std::move(l)));
		}
		//msg.setPayload(data_t("some string"));
		//logger.info("Sending msg: %s", to_string(msg).c_str());
		deliverMessageTowardsTransport(msg);
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
/** Export this transport */
SAG_DECLARE_CONNECTIVITY_TRANSPORT_CLASS(RustTransport)

}} // apamax.rust


void rust_send_msg_towards_host(void* ptr, sag_underlying_message_t* msg) {
	apamax::rust::RustTransport* t = reinterpret_cast<apamax::rust::RustTransport*>(ptr);
	t->towardsHost(reinterpret_cast<Message*>(msg));
}