/*
 * $Copyright (c) 2019-2020 Software AG, Darmstadt, Germany and/or Software AG USA Inc., Reston, VA, USA, and/or its subsidiaries and/or its affiliates and/or their licensors.$
 * Use, reproduction, transfer, publication or disclosure is prohibited except as specifically provided for in your License Agreement with Software AG
 * $Revision: 321386 $ $Date: 2017-12-14 10:26:04 +0000 (Thu, 14 Dec 2017) $
 */
#ifndef _RUST_CPP_INTERFACE_H_
#define _RUST_CPP_INTERFACE_H_

extern "C"
{
	// rust to c++ converter functions
	sag_underlying_data_t *create_cpp_data_t_empty();
	sag_underlying_data_t *create_cpp_data_t_bool(bool val);
	sag_underlying_data_t *create_cpp_data_t_int64(int64_t val);
	sag_underlying_data_t *create_cpp_data_t_double(double val);
	sag_underlying_data_t *create_cpp_data_t_string(const char *s, size_t len);
	sag_underlying_data_t *create_cpp_data_t_buffer(const uint8_t *buf, size_t size);
	sag_underlying_vector_t *create_cpp_list_t_with_capacity(int64_t capacity);
	void append_to_list_t(sag_underlying_vector_t *l, sag_underlying_data_t *d);
	sag_underlying_data_t *create_cpp_data_t_list_t(sag_underlying_vector_t *val);
	sag_underlying_map_t *create_cpp_map_t();
	void insert_into_map_t(sag_underlying_map_t *m, sag_underlying_data_t *key, sag_underlying_data_t *value);
	sag_underlying_data_t *create_cpp_data_t_map_t(sag_underlying_map_t *val);
	sag_underlying_message_t *create_cpp_message_t(sag_underlying_data_t *payload, sag_underlying_map_t *metadata);
	void free_cpp_message_t(sag_underlying_message_t *m);
}

#endif