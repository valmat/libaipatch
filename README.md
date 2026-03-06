# libaipatch

[placeholder: readme is coming here]

`libaipatch` is a small Rust-based static library that applies `codex` text patches
(`*** Begin Patch` ... `*** End Patch`) through a stable C ABI.

The v1 surface currently provides:
- `aipatch_check` — dry-run validation without filesystem writes;
- `aipatch_apply` — apply after full validation succeeds;
- `aipatch_result_free` — free ABI-allocated result messages;
- `aipatch_version` and `aipatch_abi_version`.

The canonical ABI accepts exactly two logical inputs:
- `patch` — UTF-8 patch text in `codex` format;
- `root_dir` — UTF-8 filesystem root relative to which patch paths are resolved.

## Build

Build the library in release mode:

```sh
cd rust
cargo build --release
```

Main build artifacts:
- static library: `rust/target/release/libaipatch.a`
- public header: `include/aipatch.h`

For development and regression checks:

```sh
cd rust
cargo test
```

## Link From C/C++

Minimal example:

```c
#include "aipatch.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    const char* patch =
        "*** Begin Patch\n"
        "*** Add File: hello.txt\n"
        "+hello from libaipatch\n"
        "*** End Patch\n";
    const char* root_dir = "/tmp/aipatch-demo";

    aipatch_result result = {0};
    int rc = aipatch_check(patch, strlen(patch), root_dir, strlen(root_dir), &result);
    if (rc != 0) {
        fprintf(stderr, "ABI failure: %d\n", rc);
        return 1;
    }
    if (result.code != AIPATCH_OK) {
        fprintf(stderr, "check failed: %.*s\n", (int)result.message_len, result.message);
        aipatch_result_free(&result);
        return 1;
    }
    aipatch_result_free(&result);

    rc = aipatch_apply(patch, strlen(patch), root_dir, strlen(root_dir), &result);
    if (rc != 0) {
        fprintf(stderr, "ABI failure: %d\n", rc);
        return 1;
    }
    if (result.code != AIPATCH_OK) {
        fprintf(stderr, "apply failed: %.*s\n", (int)result.message_len, result.message);
        aipatch_result_free(&result);
        return 1;
    }

    printf("success: %.*s", (int)result.message_len, result.message);
    aipatch_result_free(&result);
    return 0;
}
```

Example compile command on Linux:

```sh
cc demo.c -Iinclude rust/target/release/libaipatch.a -o demo
```

If your toolchain requires explicit system libraries when linking a Rust staticlib,
add the usual platform libraries for your environment.

## ABI Usage Notes

- Always check the C function return value first.
- If it is `0`, then inspect `aipatch_result.code`.
- `result.message` is either `NULL` or a null-terminated UTF-8 string.
- `result.message_len` is the payload length in bytes without the trailing `\0`.
- Free any non-empty result with `aipatch_result_free`.
- `aipatch_result_free` is safe to call repeatedly on the same cleared result.

## Current Behavior

Current implementation includes:
- `Add File`, `Delete File`, `Update File`, `Move to`;
- codex-compatible trailing newline behavior for resulting files;
- path validation relative to `root_dir`;
- rejection of absolute paths and path traversal outside `root_dir`;
- text-only UTF-8 file handling in v1.

The repository also contains portable scenario fixtures imported from `codex`
under `tests/fixtures/scenarios` and attribution metadata under
`third_party/codex_apply_patch`.

## License and Borrowed Files

This repository contains an active prototype implementation of `libaipatch`.

The project is prepared to be licensed under Apache License 2.0, and the root
`LICENSE` file contains the Apache 2.0 license text.

This scaffold already includes a limited set of files copied from OpenAI Codex,
specifically from `codex-rs/apply-patch`, plus the portable scenario fixtures
used by that component. Those borrowed files remain clearly attributed in
`third_party/codex_apply_patch/SOURCES.md` and
`third_party/codex_apply_patch/NOTICE.md`.

The copied upstream materials serve as compatibility references and fixture sources for the implementation described in `apply_patch_abi.md` and `implementation_plan.md`.
