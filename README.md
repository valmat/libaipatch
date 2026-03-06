# libaipatch

`libaipatch` is a small Rust-based static library that applies `codex` text
patches (`*** Begin Patch` ... `*** End Patch`) through a stable C ABI.

The project is designed for agent-driven and FFI-friendly workflows where a
model produces patch text and a host application validates or applies it without
embedding the full Codex codebase.

## Goals

`libaipatch` v1 is built around a narrow, practical contract:

- accept patch text in the native `codex` patch format;
- expose a compact C ABI that is easy to call from other languages;
- work from prebuilt artifacts instead of requiring a Rust toolchain at runtime;
- keep path validation and patch applicability checks inside the library;
- support an explicit dry-run validation mode before filesystem writes.

## Current v1 Surface

The current ABI exports:

- `aipatch_check` — dry-run validation without filesystem writes;
- `aipatch_apply` — validate first, then apply changes to disk;
- `aipatch_result_free` — free ABI-owned result messages;
- `aipatch_version` — return library version string;
- `aipatch_abi_version` — return ABI major version.

The canonical ABI accepts exactly two logical inputs:

- `patch` — UTF-8 patch text in `codex` format;
- `root_dir` — UTF-8 filesystem root relative to which patch paths are resolved.

## Supported Patch Features

`libaipatch` v1 supports the core `codex` patch operations:

- `*** Add File: <path>`
- `*** Delete File: <path>`
- `*** Update File: <path>`
- `*** Move to: <new_path>` inside `Update File`

It also preserves the main text-level behaviors expected from the upstream
implementation:

- `@@` and `@@ <context>` chunk markers;
- optional missing `@@` for the first update chunk when upstream logic permits it;
- `*** End of File` markers;
- fuzzy context matching during updates;
- codex-compatible trailing newline behavior for resulting text files.

For a more practical patch-format overview, see `docs/patch_format.md`.

## Repository Layout

Key directories in this repository:

- `rust/` — Rust crate that builds the library and contains tests;
- `include/` — public C header (`aipatch.h`);
- `docs/` — project notes and patch-format documentation;
- `tests/` — fixture data for compatibility and scenario tests;
- `third_party/codex_apply_patch/` — attribution and upstream reference material;
- `build/` — Debian packaging script and metadata.

## Build

Build the library in release mode:

```sh
cd rust
cargo build --release
```

Main build artifacts:

- static library: `rust/target/release/libaipatch.a`
- public header: `include/aipatch.h`

Run tests:

```sh
cd rust
cargo test
```

## Bindings

Additional bindings live under `bindings/`:

- `bindings/cpp/` — header-only C++ wrapper, example, and smoke-test;
- `bindings/D/` — D module, example, and smoke-test;
- `bindings/Makefile` — convenience targets for building and running binding checks.

Useful commands:

```sh
cd bindings
make cpp-test
make d-test-dmd
make d-test-ldc2
make all
```

## Debian Package

The repository includes a packaging helper for Debian-based systems:

```sh
cd build
./build.sh
```

This produces a package like:

```text
libaipatch-dev_<version>_<arch>.deb
```

The package currently installs:

- `usr/lib/libaipatch.a`
- `usr/include/aipatch.h`
- `usr/share/doc/libaipatch-dev/README.md.gz`
- `usr/share/doc/libaipatch-dev/docs/agent_notes.md.gz`
- `usr/share/doc/libaipatch-dev/docs/patch_format.md.gz`

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

Depending on the toolchain and platform, linking a Rust `staticlib` may require
additional system libraries.

## ABI Usage Notes

Important calling conventions:

- always check the C function return value first;
- if it is `0`, then inspect `aipatch_result.code`;
- `result.message` is either `NULL` or a null-terminated UTF-8 string;
- `result.message_len` is the payload length in bytes without the trailing `\0`;
- free any owned result message with `aipatch_result_free`;
- `aipatch_result_free` is safe to call repeatedly on the same cleared result.

ABI error categories currently exposed through `aipatch_result.code`:

- `AIPATCH_OK`
- `AIPATCH_INVALID_ARGUMENT`
- `AIPATCH_PARSE_ERROR`
- `AIPATCH_IO_ERROR`
- `AIPATCH_PATCH_CONFLICT`
- `AIPATCH_PATH_VIOLATION`
- `AIPATCH_UNSUPPORTED`
- `AIPATCH_INTERNAL_ERROR`

## Validation and Apply Semantics

The engine operates in two phases:

1. validation phase
   - parse patch text;
   - validate paths relative to `root_dir`;
   - read the required filesystem state;
   - verify patch applicability and compute resulting contents.
2. commit phase
   - perform writes only after the full validation phase succeeds.

Practical consequences:

- `aipatch_check` never writes to disk;
- `aipatch_apply` does not begin writing if the patch is invalid or conflicts;
- v1 does not promise global rollback once commit phase has already started.

## Path and Text Policy

`libaipatch` keeps path policy inside the library rather than pushing it to the
caller.

The current implementation rejects:

- absolute patch paths;
- lexical traversal outside `root_dir` such as `../..`;
- invalid `Move to` destinations under the same policy.

Text model in v1:

- patch input must be UTF-8;
- `root_dir` input must be UTF-8;
- files are treated as text UTF-8 files;
- binary or non-UTF-8 workflows are out of scope for v1.

The implementation is intended to block ordinary path traversal, but it is not a
full hardening layer against every possible symlink or filesystem race corner
case.

## Upstream Compatibility, Attribution, and Borrowed Files

This repository contains a focused implementation derived from logic in OpenAI
Codex, especially `codex-rs/apply-patch`, while intentionally avoiding a build
or runtime dependency on the full upstream project.

Important: this project does include files and fixture materials copied or
adapted from the Codex repository. That is intentional, documented, and kept
visible to preserve both technical traceability and honest attribution.

Borrowed materials and fixture provenance are documented in:

- `third_party/codex_apply_patch/SOURCES.md`
- `third_party/codex_apply_patch/NOTICE.md`

These records describe:

- which files were taken from upstream Codex;
- which upstream revision they came from;
- what local modifications were made;
- why each borrowed file is included in this repository.

The copied upstream materials serve as compatibility references and fixture
sources for the implementation described in `apply_patch_abi.md` and
`implementation_plan.md`.

## License and Legal Notes

The project is prepared to be licensed under Apache License 2.0, and the root
`LICENSE` file contains the Apache 2.0 license text.

The borrowed Codex-derived materials remain explicitly attributed through the
files under `third_party/codex_apply_patch/`, so that the repository preserves
both legal clarity and an honest record of where the implementation ideas and
reference files came from.

## Additional Documentation

- `docs/agent_notes.md` — integration notes for agent and tool authors;
- `docs/patch_format.md` — accepted patch format with examples.

