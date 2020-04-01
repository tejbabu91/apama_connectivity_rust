#ifndef _RUST_CPP_INTERFACE_H_
#define _RUST_CPP_INTERFACE_H_

extern "C" {
	// rust to c++ converter functions
	sag_underlying_data_t* create_cpp_data_t_empty();
	sag_underlying_data_t* create_cpp_data_t_bool(bool val);
	sag_underlying_data_t* create_cpp_data_t_int64(int64_t val);
	sag_underlying_data_t* create_cpp_data_t_double(double val);
	sag_underlying_vector_t* create_cpp_list_t_with_capacity(int64_t capacity);
	void append_to_list_t(sag_underlying_vector_t *l, sag_underlying_data_t *d);
	sag_underlying_data_t* create_cpp_data_t_list_t(sag_underlying_vector_t *val);
	sag_underlying_map_t* create_cpp_map_t();
	void insert_into_map_t(sag_underlying_map_t *m, sag_underlying_data_t *key, sag_underlying_data_t *value);
	sag_underlying_data_t* create_cpp_data_t_map_t(sag_underlying_map_t *val);
}



#endif