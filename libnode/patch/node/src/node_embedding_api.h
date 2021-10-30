#ifndef NODE_EMBEDDING_API_H
#define NODE_EMBEDDING_API_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
	int process_argc;
	const char* const * process_argv;
        void* napi_reg_func; // napi_addon_register_func
} node_options_t;

typedef struct {
	int exit_code;
	char* error; // null-terminated. Caller is responsible for calling free() on it
} node_run_result_t;

node_run_result_t node_run(node_options_t);

#ifdef __cplusplus
}
#endif

#endif
