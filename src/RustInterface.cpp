/*
 * $Copyright (c) 2019-2020 Software AG, Darmstadt, Germany and/or Software AG USA Inc., Reston, VA, USA, and/or its subsidiaries and/or its affiliates and/or their licensors.$
 * Use, reproduction, transfer, publication or disclosure is prohibited except as specifically provided for in your License Agreement with Software AG
 * $Revision: 321386 $ $Date: 2017-12-14 10:26:04 +0000 (Thu, 14 Dec 2017) $
 */

#include <sag_connectivity_plugins.hpp>
// #include <sag_connectivity_threading.h>
#include <vector>
#include<sstream>
#include<iostream>
#include <cmath>
#include "RustInterface.h"

using namespace com::softwareag::connectivity;

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

sag_underlying_data_t* create_cpp_data_t_string(const char* s) {
	return reinterpret_cast<sag_underlying_data_t*>(new data_t(s));
}

sag_underlying_data_t* create_cpp_data_t_buffer(const uint8_t* buf, size_t size) {
	buffer_t b(size);
	for(size_t i=0;i<size;i++) {
		b[i] = buf[i];
	}
	return reinterpret_cast<sag_underlying_data_t*>(new data_t(std::move(b)));
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

sag_underlying_message_t* create_cpp_message_t(sag_underlying_data_t *payload, sag_underlying_map_t *metadata) {
	data_t *d = reinterpret_cast<data_t*>(payload);
	map_t *m = reinterpret_cast<map_t*>(metadata);
	auto retval = reinterpret_cast<sag_underlying_message_t*>(new Message(std::move(*d), std::move(*m)));
	delete d;
	delete m;
	return retval;
}
