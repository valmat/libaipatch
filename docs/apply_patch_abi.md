# `libaipatch` Implementation Spec

This document defines the practical implementation specification for the `libaipatch.a` library.
Its goal is to give a clear contract so that developers can start coding without repeating the architecture discussion.

---

## 1. Project goal

We need to implement a standalone static library `libaipatch.a` that:

- accepts a patch in the `codex` text format (`*** Begin Patch ... *** End Patch`);
- exports a simple C ABI;
- can be used from any language via FFI;
- does not require `codex` as a dependency, submodule, or workspace component;
- does not require a Rust toolchain on the consumer side;
- leaves the consumer only ready-made artifacts: `libaipatch.a` and a C header;
- enforces a mandatory `dry-run` (`check`) before writing to disk.

The library is aimed first of all at agent-driven console workflows, where the model generates the patch itself.

---

## 2. Why this format

### 2.1 Why not unified diff

A practical observation: it is noticeably harder for agents to generate and maintain classic diff formats reliably, especially when they must count line ranges precisely and keep the context in sync.

### 2.2 Why not V4A from the Responses API

OpenAI has at least two patch approaches:

- the structured `apply_patch` tool in the Responses API (operation objects and V4A diff);
- the text `codex` format: `*** Begin Patch ... *** End Patch`.

For this library we choose the second option because:

- it fits a familiar agent text workflow;
- it is easier to present to the model as a tool;
- there is already a working implementation in `codex`;
- it avoids an extra conversion layer between tool-call objects and a text patch.

### 2.3 Canonical principle

The priority is **maximum native usability for the agent**, not the smallest possible host interface.

That is why the canonical input format of the library is fixed as the **text `codex` patch format**.

---

## 3. Non-goals for v1

The first version of the library **does not include these as required goals**:

- a public API for shell/heredoc/argv detection;
- a public API that exposes parser AST/hunk/chunk structures;
- a public API for preview/unified diff;
- a public API for a whole-patch in-memory virtual filesystem mode;
- full global transactional behavior for multi-file operations with rollback of all already written files;
- support for binary/non-UTF-8 files.

---

## 4. Scope v1

### 4.1 Supported patch language

v1 must support the native `codex` patch format without intentionally removing its basic operations:

- `*** Add File: <path>`
- `*** Delete File: <path>`
- `*** Update File: <path>`
- `*** Move to: <new_path>`

The key format features must also be preserved:

- `@@` and `@@ <context>` chunk markers;
- the ability for the first chunk in `Update File` to have no `@@`, if the original logic allows it;
- `*** End of File`;
- stable fuzzy context matching;
- behavior compatible with current `codex` regarding the trailing newline.

Note: the current `codex` implementation deterministically adds a trailing newline to the resulting file. In library v1 this behavior is kept for compatibility. A separate mode to control the final newline precisely can be considered later as a contract extension.

### 4.2 Supported ABI modes

In v1 only two working modes are mandatory:

- `check` — dry-run without writing to disk;
- `apply` — actually apply the patch to the filesystem.

### 4.3 Canonical ABI input

The canonical ABI accepts:

- a UTF-8 patch string;
- `root_dir`, which is used as the base directory for resolving paths from the patch.

The canonical ABI **does not accept** a separate `file_name` as the main input.
If a convenient single-file wrapper appears later, it is considered optional and not the canonical API.

---

## 5. Project layout

Expected project composition:

- `rust/` (or a similar directory) with an isolated Rust implementation;
- `include/` (or a similar directory) with a public C header;
- `tests/` with ABI tests, integration smoke tests, and ported scenarios;
- optionally `bindings/` or `examples/bindings/` with example bindings for C++/D;
- `README`/`LICENSE`/attribution for borrowed code.

Minimal expected logical layers:

1. Patch engine layer
   - parser;
   - seek/fuzzy matching;
   - change computation.
2. Safety/orchestration layer
   - path policy;
   - `check` pipeline;
   - `apply` pipeline;
   - safe disk writes.
3. C ABI layer
   - exported functions;
   - result object;
   - memory management.

---

## 6. What is borrowed from `codex`

### 6.1 Required candidates to port

The most useful parts from `codex-rs/apply-patch` are:

- `src/parser.rs`
- `src/seek_sequence.rs`
- relevant parts of `src/lib.rs`
- test scenarios from `tests/fixtures/scenarios`
- if needed, the text description of the patch format from `apply_patch_tool_instructions.md`

### 6.2 What we do not port in v1

These parts are not needed for canonical ABI v1:

- `src/invocation.rs`
- shell/heredoc parsing;
- `tree-sitter`-dependent logic;
- the CLI-specific layer `standalone_executable`.

### 6.3 Porting principle

Only what the library really needs is ported.
There must be no dependency on the full `codex` project.

---

## 7. Dependencies policy

### 7.1 Required dependencies for v1

Only dependencies that are truly needed for the canonical ABI are allowed.

At the start we expect this split:

- required core: parser + apply logic;
- optional: `similar`, if we need a richer preview in the future;
- not required for v1 ABI: `tree-sitter`, `tree-sitter-bash`.

### 7.2 Priority

We prefer a minimal amount of custom Rust code, but not at the expense of:

- path safety;
- dry-run/apply orchestration;
- the C ABI;
- safe writing.

In other words: custom Rust code is allowed when it fulfills the library contract and safety requirements, and does not just copy existing `codex` core code without a real need.

---

## 8. Canonical C ABI

### 8.1 Public types

```c
typedef struct {
    int code;
    char* message;
    size_t message_len;
} aipatch_result;
```

### 8.2 Public functions v1

```c
int aipatch_check(
    const char* patch,
    size_t patch_len,
    const char* root_dir,
    size_t root_dir_len,
    aipatch_result* out
);

int aipatch_apply(
    const char* patch,
    size_t patch_len,
    const char* root_dir,
    size_t root_dir_len,
    aipatch_result* out
);

void aipatch_result_free(aipatch_result* result);

const char* aipatch_version(void);
int aipatch_abi_version(void);
```

### 8.3 General ABI semantics

- `patch` and `root_dir` are treated as UTF-8 strings;
- lengths are passed explicitly, not via `strlen`;
- `out` is required and is filled by the library;
- memory for `out->message`, if allocated, must be freed only via `aipatch_result_free`.

### 8.4 Two-level status model

The functions return **two levels of result**, and they are **not the same thing**:

1. the C function return value
   - describes whether the ABI call itself was valid and whether the library runtime layer worked;
   - typical success path: the function returns `0`, then the caller checks `out->code`.
2. `aipatch_result.code`
   - describes the application-level result: success, parse error, conflict, I/O error, etc.

Recommended way for the caller to read the result:

1. check the return value first;
2. if the return value indicates ABI-level success, check `out->code`;
3. if `out->code` is non-zero, use `out->message` as diagnostics.

This split must be kept in order to:

- not mix FFI/ABI-level errors with patch engine errors;
- keep a simple contract for the host;
- allow safe bindings with RAII.

Note: the idea to remove `code` from `aipatch_result` and use only the return value was considered. This spec deliberately keeps the two-level model to avoid mixing infrastructure errors of ABI/runtime with the application result of `check/apply`.

---

## 9. `aipatch_result` contract

### 9.1 Required rules

- for every call, `out` must be set to a consistent state;
- `message` may be `NULL`;
- `message_len == 0` if `message == NULL`;
- if `message != NULL`, the string must be UTF-8 encoded;
- if `message != NULL`, the string must be null-terminated for C convenience;
- `message_len` must describe the byte length of the useful string without the ending `\0`;
- `aipatch_result_free` must be safe when called multiple times on an already cleared result;
- `aipatch_result_free` must be safe for `message == NULL`.

### 9.2 Minimal meaning of `message`

- on error, `message` contains human-readable diagnostics;
- on many common errors, `message` may also include stable machine-oriented fields such as `tag:`, `hint:`, and `detail:`;
- on success, `message` may be empty or contain a short summary;
- a detailed preview in `message` is not required;
- the caller may use either `message_len` or null-termination, but the canonical length source is `message_len`.

---

## 10. Error codes policy

A full detailed table of codes can be refined by the implementer, but in v1 there must be **at least** these stable categories:

- `AIPATCH_OK`
- `AIPATCH_INVALID_ARGUMENT`
- `AIPATCH_PARSE_ERROR`
- `AIPATCH_IO_ERROR`
- `AIPATCH_PATCH_CONFLICT`
- `AIPATCH_PATH_VIOLATION`
- `AIPATCH_UNSUPPORTED`
- `AIPATCH_INTERNAL_ERROR`

The exact mapping from internal Rust errors to these categories is up to the implementer, as long as it is predictable and documented.

The idea is:

- not to overload the spec with premature details;
- not to block implementation choices;
- but still to define a minimal stable external contract.

---

## 11. Canonical semantics of `check` and `apply`

Extra convention for the return value:

- `0` means the ABI call was successful and `out` contains the application result;
- a non-zero value means an ABI/runtime error where `out->code` must not be treated as the only source of truth.

### 11.1 `aipatch_check`

`aipatch_check(...)` must:

- parse the patch;
- validate paths relative to `root_dir`;
- verify that the patch can be applied;
- compute the final changes as much as needed for a reliable check;
- not write to disk;
- return human-readable diagnostics on error.

### 11.2 `aipatch_apply`

`aipatch_apply(...)` must:

- perform the same validation as `check`;
- not start writing to disk if the patch is invalid or cannot be applied;
- write changes only after a fully successful check phase;
- if an error happens before the commit phase, not change the filesystem.

### 11.3 What v1 does not have to guarantee

v1 **does not have to** guarantee full global rollback transactionality for the whole multi-file operation if a rare system error happens during the commit phase.

But v1 must guarantee a weaker and mandatory property:

- if the patch fails `check`, `apply` must not start writing.

---

## 12. Write semantics

### 12.1 General principle

Disk writes start only after a successful full check phase.

### 12.2 `Update File`

Recommended semantics:

1. read the original file;
2. compute the new content in memory;
3. write the new content to a temporary file next to the target;
4. replace the original file via `rename`, where possible.

### 12.3 `Add File`

Recommended semantics:

1. validate that the operation is allowed;
2. create parent directories if needed;
3. write only during the commit phase;
4. if possible, use the same safe temporary-file + `rename` scheme.

### 12.4 `Delete File`

Recommended semantics:

- delete the file only after a successful global check phase;
- do not change the filesystem before the commit phase.

### 12.5 `Move to`

Recommended semantics:

- treat it as a destination write operation plus source deletion;
- the destination path must pass the same path validation;
- the source must not be deleted before the destination is prepared successfully.

---

## 13. Path policy

Path policy is a required part of the library and **must not be left to the caller**.

### 13.1 Why it must live in the library

The canonical ABI accepts only `patch + root_dir`.
So the library itself:

- parses the patch;
- extracts paths;
- resolves them;
- opens/creates/deletes files.

The caller has no reliable way to safely intercept path traversal if the library does not handle it.

### 13.2 Minimal path policy v1

In v1 at least these rules must be enforced:

- paths in the patch are treated as relative to `root_dir`;
- absolute paths in the patch are forbidden;
- paths that escape `root_dir` after normalization are forbidden;
- the same rules apply to the `Move to` destination.

### 13.3 Goal of the path policy

We must prevent scenarios like:

- `../.ssh/id_rsa`
- `../../outside/of/root`
- other forms of path traversal outside the allowed root.

### 13.4 What can be postponed

A very strict hardening model against all race/symlink corner cases can be added in later versions.
But basic protection against path traversal is mandatory already in v1.

---

## 14. Text and encoding policy

Library v1 is text-based.

This means:

- the patch must be UTF-8;
- paths must be UTF-8;
- files handled by the library are treated as UTF-8 text files;
- binary/non-UTF-8 support is not required in v1.

If the library cannot read a target file as UTF-8, it must return a diagnosable error in the appropriate category.

---

## 15. Platform scope

v1 targets a Linux/POSIX-first environment.

This means:

- Linux is the main target platform;
- POSIX-like systems may be supported when possible;
- Windows is not a target platform for v1 and does not define requirements for path semantics, path encoding, or file operations in the first version.

If Windows support is added later, it must be described as a separate contract extension.

---

## 16. Fuzzy matching expectations

When porting the core from `codex`, the context search logic must stay as close as reasonably possible.

Expected properties:

- try exact match first;
- then use a more lenient whitespace-based comparison;
- keep robustness against typical agent-authored patch variance;
- keep support for `*** End of File`;
- no required need to count line numbers.

---

## 17. Thread-safety and global state

In the canonical v1 design there should be no global mutable state required for the library to work.

Practical requirements for the implementation:

- do not introduce global mutable state unless truly necessary;
- design `check` and `apply` as re-entrant calls with independent state on the stack/in local objects;
- if the final implementation really uses no global mutable state, this can be promised in `README` as thread-safe behavior for independent calls.

Thread-safety is not an explicit goal of v1, but if it comes naturally without making the architecture heavier, it is a desirable property.

---

## 18. Test strategy

### 18.1 General principle

Tests from `codex` should be ported as widely as possible, but only for the functionality that is explicitly supported in v1.

### 18.2 What we port

First of all we should bring over:

- `tests/fixtures/scenarios` from `codex-rs/apply-patch`;
- related scenarios that cover `Add/Delete/Update/Move`;
- edge cases for EOF, empty chunk, first chunk without `@@`, trailing newline, and path handling.

### 18.3 Required minimum

Required test groups for v1:

- parse success / parse failure;
- `check` success without writing to disk;
- `check` failure with clear diagnostics;
- `apply` success;
- `apply` after a successful `check`;
- `apply` does not change the filesystem if the patch is invalid or cannot be applied;
- path traversal rejection;
- `Add/Delete/Update/Move` within the supported scope;
- key compatible edge cases from `codex` fixtures.

### 18.4 Optional

Optionally it is useful to add:

- smoke tests for C++ bindings if the project includes `bindings/` or `examples/bindings/`;
- smoke tests for D bindings if the project includes `bindings/` or `examples/bindings/`;
- a regression suite for updating borrowed code from `codex`.

---

## 19. Implementation structure

In practice we expect this implementation plan:

1. Port the parser and seek logic from `codex`.
2. Extract a patch application core **without CLI glue**.
3. Add our own safety layer:
   - path validation;
   - `check` pipeline;
   - `apply` pipeline;
   - safe write semantics.
4. Add the C ABI layer.
5. Add ported scenario tests and ABI smoke tests.

---

## 20. What can be added later

These features are reasonable to leave as future work:

- a richer preview API;
- a structured change list in the ABI response;
- unified diff preview;
- a single-file convenience API `patch + file_name`;
- an in-memory virtual filesystem mode;
- stronger hardening for symlink/race cases;
- additional language bindings beyond the C ABI.

---

## 21. Appendix A — Patch format summary

A short cheat sheet for the supported patch format:

```text
*** Begin Patch
*** Add File: path/to/file
+line 1
+line 2
*** Update File: path/to/other
@@
-context line
+new line
*** Delete File: path/to/third
*** End Patch
```

Supported operations:

- `Add File`
- `Delete File`
- `Update File`
- `Move to` inside `Update File`

Key format properties:

- does not require line numbers;
- uses context lines and pattern matching;
- works well for agent-authored edits;
- allows the model to construct patches in a fairly natural text form.

---

## 22. Appendix B — What to tell an agent

If you need a short explanation for a system/agent about what patch format is expected, a description like this is enough:

- Use the patch format `*** Begin Patch ... *** End Patch`.
- For new files use `*** Add File: path` and lines starting with `+`.
- For deleting files use `*** Delete File: path`.
- For changing files use `*** Update File: path`, then `@@` and context/replacement lines with prefixes ` `, `-`, `+`.
- Do not count line numbers.
- Try to make small, local, and precise changes.

This appendix is supportive. The primary focus of the project is the library and its ABI, not prompt engineering.

---

## 23. Final position

The fixed canonical project model:

- input: native text `codex` patch;
- ABI: `patch + root_dir`;
- modes: `check` and `apply`;
- safety: mandatory path policy + dry-run before write;
- borrowing from `codex`: selective, without a dependency on the full project;
- tests: ported `codex` scenarios + our own ABI/safety tests.

This contract is enough to start implementation.
