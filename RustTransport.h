#ifndef INCLUDE__RUST_SUPPORT__
#define INCLUDE__RUST_SUPPORT__
/*
 * $Copyright (c) 2019 Software AG, Darmstadt, Germany and/or Software AG USA Inc., Reston, VA, USA, and/or its subsidiaries and/or its affiliates and/or their licensors.$
 * Use, reproduction, transfer, publication or disclosure is prohibited except as specifically provided for in your License Agreement with Software AG
 * $Revision: 321386 $ $Date: 2017-12-14 10:26:04 +0000 (Thu, 14 Dec 2017) $
 */

#include <sag_connectivity_plugins.hpp>

namespace apamax {
namespace rust {

/**
 *  GoTransport which initializes go runtime and handle communication to/from go transport
 */
class RustTransport: public com::softwareag::connectivity::AbstractSimpleTransport
{

	void * rustTransport;
public:
	/** Create the transport and check we have the correct (no) config */
	RustTransport(const TransportConstructorParameters &params);
	~RustTransport();

	/** Start the request-handling thread */
	virtual void start();

	/** Stop the plugin and wait for the request-handling thread */
	virtual void shutdown();

	/** Parse the request and queue it for later servicing */
	virtual void deliverMessageTowardsTransport(com::softwareag::connectivity::Message &m) override;

    void towardsHost(com::softwareag::connectivity::Message *m);

	virtual void hostReady();
};

}} // apamax.golang
#endif