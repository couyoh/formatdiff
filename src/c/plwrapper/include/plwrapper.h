#ifndef plwrapper_H
#define plwrapper_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

using pl_plwrapper = struct pl_plwrapper;
using pl_result = struct {
    const char *path;
    const char *type_name;
    const char *value;
    uint64_t offset;
    uint64_t size;
};

pl_plwrapper *pl_create(void);
void pl_destroy(pl_plwrapper *rt);
void pl_add_include_path(pl_plwrapper *rt, const char *path);
void pl_set_max_eval_depth(pl_plwrapper *rt, uint64_t depth);
int pl_execute_file(pl_plwrapper *rt, const char *pattern_path,
                    const char *data_path);
void pl_abort(pl_plwrapper *rt);
const char *pl_last_error(pl_plwrapper *rt);
size_t pl_field_count(pl_plwrapper *rt);
const pl_result *pl_field_at(pl_plwrapper *rt, size_t index);

#ifdef __cplusplus
}
#endif

#endif
