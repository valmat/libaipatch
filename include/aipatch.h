#ifndef AIPATCH_H
#define AIPATCH_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ------------------------------------------------------------------ */
/* Error codes (aipatch_result.code)                                   */
/* ------------------------------------------------------------------ */

#define AIPATCH_OK               0
#define AIPATCH_INVALID_ARGUMENT 1
#define AIPATCH_PARSE_ERROR      2
#define AIPATCH_IO_ERROR         3
#define AIPATCH_PATCH_CONFLICT   4
#define AIPATCH_PATH_VIOLATION   5
#define AIPATCH_UNSUPPORTED      6
#define AIPATCH_INTERNAL_ERROR   7

/* ------------------------------------------------------------------ */
/* Result struct                                                        */
/* ------------------------------------------------------------------ */

/**
 * Result returned by aipatch_check and aipatch_apply.
 *
 * Two-level status model:
 *  1. The return value of the C function describes the ABI-level call
 *     result (0 = success, non-zero = catastrophic ABI failure, e.g. null out).
 *  2. aipatch_result.code describes the application-level result.
 *
 * message:
 *  - May be NULL (on success, message may be empty or NULL).
 *  - Always null-terminated if non-NULL.
 *  - message_len gives the payload length in bytes EXCLUDING the '\0'.
 *  - Must be freed exactly once via aipatch_result_free().
 */
typedef struct {
    int    code;
    char*  message;
    size_t message_len;
} aipatch_result;

/* ------------------------------------------------------------------ */
/* Public API                                                           */
/* ------------------------------------------------------------------ */

/**
 * Validate and dry-run a patch against the filesystem rooted at root_dir.
 *
 * Does NOT write anything to disk.
 *
 * @param patch        UTF-8 encoded patch text.
 * @param patch_len    Length of patch in bytes.
 * @param root_dir     UTF-8 path to the root directory.
 * @param root_dir_len Length of root_dir in bytes.
 * @param out          Non-null pointer to an aipatch_result to be filled.
 * @return 0 on ABI success (check out->code for application result).
 *         -1 if out is NULL.
 */
int aipatch_check(
    const char* patch,
    size_t      patch_len,
    const char* root_dir,
    size_t      root_dir_len,
    aipatch_result* out
);

/**
 * Apply a patch to the filesystem rooted at root_dir.
 *
 * Validates first; writes only after full successful validation.
 *
 * @param patch        UTF-8 encoded patch text.
 * @param patch_len    Length of patch in bytes.
 * @param root_dir     UTF-8 path to the root directory.
 * @param root_dir_len Length of root_dir in bytes.
 * @param out          Non-null pointer to an aipatch_result to be filled.
 * @return 0 on ABI success (check out->code for application result).
 *         -1 if out is NULL.
 */
int aipatch_apply(
    const char* patch,
    size_t      patch_len,
    const char* root_dir,
    size_t      root_dir_len,
    aipatch_result* out
);

/**
 * Free memory allocated in an aipatch_result.
 *
 * Safe to call on a result with message == NULL.
 * Safe to call multiple times (idempotent).
 */
void aipatch_result_free(aipatch_result* result);

/**
 * Return a null-terminated string with the library version.
 * The returned pointer is valid for the lifetime of the process.
 */
const char* aipatch_version(void);

/**
 * Return the ABI major version number.
 */
int aipatch_abi_version(void);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* AIPATCH_H */
