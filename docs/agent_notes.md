# Agent Notes for `libaipatch`

This document is aimed at agent and tool authors who want to feed `libaipatch`
with `codex`-style text patches.

## What the library does

`libaipatch` applies patches written in the textual `codex` format:

```text
*** Begin Patch
...
*** End Patch
```

The v1 ABI exposes two primary operations:

- `aipatch_check` — parse and validate a patch without writing to disk;
- `aipatch_apply` — validate first, then apply changes to the filesystem.

The canonical ABI inputs are:

- `patch` — UTF-8 patch text;
- `root_dir` — UTF-8 filesystem root used for resolving patch paths.

## Supported hunk types

v1 supports the native `codex` patch operations used by the upstream
`codex-rs/apply-patch` implementation:

- `*** Add File: <path>`
- `*** Delete File: <path>`
- `*** Update File: <path>`
- `*** Move to: <new_path>` inside `Update File`

Update hunks support:

- `@@`
- `@@ <context>`
- context lines starting with a space (` `)
- removed lines starting with `-`
- added lines starting with `+`
- `*** End of File`

The first update chunk may omit `@@` if the upstream parser logic allows it.

## Operational model

The engine works in two phases:

1. validation phase
   - parse patch text;
   - validate patch paths relative to `root_dir`;
   - read the required files;
   - verify that each hunk can be applied;
   - compute the resulting file contents in memory.
2. commit phase
   - perform filesystem writes only after validation succeeds completely.

This means:

- `check` never writes to disk;
- `apply` does not start writing if the patch is invalid or conflicts with the
  current filesystem state.

v1 does **not** guarantee global rollback after commit phase has started.

## Path rules

All patch paths are interpreted relative to `root_dir`.

The library rejects:

- absolute paths;
- lexical path traversal outside `root_dir`, such as `../..`;
- invalid `Move to` destinations that violate the same policy.

Current v1 implementation is designed to block ordinary path traversal and keep
path policy inside the library. It is **not** a full hardening layer against all
possible symlink or filesystem race corner cases.

## Text model

`libaipatch` is a text-oriented v1 implementation.

- patch text must be valid UTF-8;
- `root_dir` must be valid UTF-8;
- target files are treated as UTF-8 text files;
- binary or non-UTF-8 file workflows are out of scope for v1 and are rejected with `AIPATCH_UNSUPPORTED`.

The implementation follows the upstream `codex` behavior of ensuring a trailing
newline in resulting text files.

## Fuzzy matching expectations

When applying update hunks, the library tries to match existing content using a
fuzzy search compatible with the extracted upstream logic.

Matching progressively relaxes comparison rules:

1. exact line match;
2. trailing-whitespace-insensitive match;
3. trim-insensitive match;
4. a final normalization pass for a small set of common Unicode punctuation and
   spacing variants.

This helps agents generate practical patches without requiring exact byte-for-byte
line context in every case.

## Error categories

The C ABI exposes stable high-level error categories through
`aipatch_result.code`:

- `AIPATCH_OK`
- `AIPATCH_INVALID_ARGUMENT`
- `AIPATCH_PARSE_ERROR`
- `AIPATCH_IO_ERROR`
- `AIPATCH_PATCH_CONFLICT`
- `AIPATCH_PATH_VIOLATION`
- `AIPATCH_UNSUPPORTED`
- `AIPATCH_INTERNAL_ERROR`

Callers should always check the C function return value first and then inspect
`aipatch_result.code`.

For machine-assisted recovery, many v1 diagnostics in `aipatch_result.message`
now follow a stable text shape with fields such as:

- `tag: ...`
- `hint: ...`
- `detail: ...`

Some errors also include additional fields like `file:`, `path:`, `root_dir:`,
`hunk:`, `expected_context:`, `expected_lines:`, or `nearest_actual:`.

## Patch authoring guidance for agents

When generating patches for this library:

- always emit the full `*** Begin Patch` / `*** End Patch` envelope;
- keep file paths relative to `root_dir`;
- use `Add File` only when the target file does not already exist;
- use `Update File` when modifying existing files;
- use `Move to` only inside an `Update File` hunk;
- include enough nearby context to make matching unambiguous;
- prefer a small number of focused hunks over one huge hunk.

Good agent patches are usually easier to apply when they:

- preserve indentation faithfully;
- avoid touching unrelated lines;
- do not rely on shell syntax, heredoc wrappers, or CLI-specific invocation forms
  as part of the canonical ABI contract.

## Non-goals for v1

The current library intentionally does not provide:

- shell or heredoc invocation parsing as public ABI;
- public parser AST ABI;
- binary patch support;
- whole-filesystem transactional rollback;
- full CLI compatibility surface of the upstream `codex` tool.
