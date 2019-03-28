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
#include <cmath>
#include "RustInterface.h"

using namespace com::softwareag::connectivity;

namespace apamax {
namespace rust {
RustTransport::RustTransport(const TransportConstructorParameters &params)
		: AbstractSimpleTransport(params)
	{
		
		logger.info("Sum from rust: %d", add(10, 20));
		rustTransport = rust_transport_create();
		logger.info("Rust transport object: %d", rustTransport);
		call_back_from_c(rustTransport);
		Data d = {19, 42};
		send_data_towards_transport(&d);
		// map_t &config = const_cast<map_t&>(params.getConfig());
		// logger.info("C++ config: %s", to_string(config).c_str());
		// std::ostringstream os;
		// apply_visitor(JsonVisitor(os), data_t(std::move(config)));
		// logger.info("Config JSON: %s", os.str().c_str());
		// std::string content = os.str();
		// //char * str = const_cast<char*>(payload.c_str());
		// go_transport_create(this, static_cast<void*>(const_cast<char*>(content.c_str())), content.size());
	}

	void RustTransport::start()
	{
		logger.info("C++ start called");
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
		logger.info("Sending msg: %s", to_string(msg).c_str());
		rust_transport_send_msg_towards(rustTransport, reinterpret_cast<sag_underlying_message_t*>(&msg));
		// go_transport_start(this);

		// char buf[11] = "HelloWorld";
		// CallIntoTransport(buf, sizeof(buf)-1);
	}

	/** Stop the plugin and wait for the request-handling thread */
	void RustTransport::shutdown()
	{
		rust_transport_shutdown(rustTransport);
	}

	/** Parse the request and queue it for later servicing */
	void RustTransport::deliverMessageTowardsTransport(Message &m)
	{
		logger.info("C++ deliverMessageTowardsTransport: %s", to_string(m).c_str());
		rust_transport_send_msg_towards(rustTransport, reinterpret_cast<sag_underlying_message_t*>(&m));
		// auto payload = get<std::string>(m.getPayload());
		// char * str = const_cast<char*>(payload.c_str());
		// go_transport_deliverMessageTowardsTransport(this, static_cast<void*>(str), payload.size());
	}

	void RustTransport::towardsHost(char* buf, int bufLen) {
		data_t buffer(buf, bufLen);
		Message m;
		m.setPayload(std::move(buffer));
		hostSide->sendBatchTowardsHost(&m, &m+1);
	}

	void RustTransport::hostReady() {
		rust_transport_hostReady(rustTransport);
	}
/** Export this transport */
SAG_DECLARE_CONNECTIVITY_TRANSPORT_CLASS(RustTransport)

}} // apamax.golang


