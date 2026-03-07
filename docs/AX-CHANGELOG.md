# Changelog

## Unreleased

### AX / Agent Experience

- Improved parse diagnostics to use a more agent-friendly `message` shape with stable fields such as `tag:`, `hint:`, and `detail:`.
- Added targeted parse hints for common malformed patch cases, including empty `Add File`, empty `Update File`, empty `Move to` updates, missing `@@`, and invalid update-line prefixes.
- Greatly improved `patch_conflict` diagnostics with structured fields such as `file:`, `hunk:`, `expected_context:`, `expected_lines:`, and `nearest_actual:`.
- Standardized many runtime and validation errors into a more machine-usable text format without changing the ABI.
- Standardized `I/O error` diagnostics to include `tag: io.error`, `hint:`, `context:`, `kind:`, and `detail:`.
- Standardized success output from `aipatch_apply` into a compact agent-friendly summary:
  - `status: ok`
  - `operations:`
  - `A <path>` / `M <path>` / `D <path>`

### Behavior changes

- `Add File` now rejects existing destination files instead of overwriting them.
- Binary-ish files and non-UTF-8 text files are now reported as `AIPATCH_UNSUPPORTED` with stable diagnostics, instead of falling through as generic I/O failures.
- Existing upstream scenario `011_add_overwrites_existing_file` is intentionally treated as a known semantic divergence from upstream fixtures.

### Documentation

- Expanded `docs/agent_notes.md` with:
  - the stable `message` field shape;
  - diagnostic tag families;
  - field conventions such as `file:` vs `path:`;
  - the new success summary format.
- Updated `docs/patch_format.md` to document:
  - `Add File` rejection when the target file already exists;
  - rejection of binary / non-UTF-8 targets.
- Updated `docs/apply_patch_abi.md` to reflect that `message` may contain stable machine-oriented fields for common errors.
- Added `todo.apply_patch.md` with a concrete migration plan for adapting the external `apply_patch.d` wrapper to the new diagnostics model.

### Tests

- Added targeted tests for:
  - new parse diagnostic tags and hints;
  - `Add File` rejection on existing files;
  - non-UTF-8 and binary target rejection;
  - richer `patch_conflict` fields;
  - `I/O error` formatting;
  - success summary shape;
  - field-consistency rules such as `parent:` + `file:` usage.
- Full Rust test suite, ABI smoke tests, and scenario tests pass after the changes.
