# Borrowed Sources from OpenAI Codex

Upstream repository: `codex`

Upstream component: `codex-rs/apply-patch`

Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`

This file records every file currently copied into `libaipatch/` from the
upstream Codex tree, as required by the project planning documents.

Each entry includes:
- upstream path in `codex`;
- upstream revision;
- local changes relative to upstream;
- reason for inclusion.

## Upstream source snapshots

### `third_party/codex_apply_patch/upstream/src/parser.rs`
- Upstream path: `codex/codex-rs/apply-patch/src/parser.rs`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the `third_party` area.
- Reason: primary reference for patch parsing and internal hunk/chunk AST.

### `third_party/codex_apply_patch/upstream/src/lib.rs`
- Upstream path: `codex/codex-rs/apply-patch/src/lib.rs`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the `third_party` area.
- Reason: primary reference for filesystem-level patch application logic and text-apply behavior.

### `third_party/codex_apply_patch/upstream/src/seek_sequence.rs`
- Upstream path: `codex/codex-rs/apply-patch/src/seek_sequence.rs`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the `third_party` area.
- Reason: primary reference for fuzzy sequence matching during patch application.

### `third_party/codex_apply_patch/upstream/src/invocation.rs`
- Upstream path: `codex/codex-rs/apply-patch/src/invocation.rs`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the `third_party` area.
- Reason: retained as an upstream reference because it is listed in the design documents as part of the main Codex apply-patch codebase, even though v1 `libaipatch` does not plan to port this layer initially.

### `third_party/codex_apply_patch/upstream/src/standalone_executable.rs`
- Upstream path: `codex/codex-rs/apply-patch/src/standalone_executable.rs`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the `third_party` area.
- Reason: retained as an upstream reference because it is listed in the design documents as part of the main Codex apply-patch codebase, even though v1 `libaipatch` does not plan to port this layer initially.

### `third_party/codex_apply_patch/upstream/apply_patch_tool_instructions.md`
- Upstream path: `codex/codex-rs/apply-patch/apply_patch_tool_instructions.md`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the `third_party` area.
- Reason: reference text for the patch format and operational instructions.

## Ported test fixtures

### `tests/fixtures/scenarios/.gitattributes`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/.gitattributes`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: kept together with the upstream fixture tree to preserve fixture semantics and text handling.

### `tests/fixtures/scenarios/001_add_file/expected/bar.md`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/001_add_file/expected/bar.md`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/001_add_file/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/001_add_file/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/002_multiple_operations/expected/modify.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/002_multiple_operations/expected/modify.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/002_multiple_operations/expected/nested/new.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/002_multiple_operations/expected/nested/new.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/002_multiple_operations/input/delete.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/002_multiple_operations/input/delete.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/002_multiple_operations/input/modify.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/002_multiple_operations/input/modify.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/002_multiple_operations/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/002_multiple_operations/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/003_multiple_chunks/expected/multi.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/003_multiple_chunks/expected/multi.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/003_multiple_chunks/input/multi.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/003_multiple_chunks/input/multi.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/003_multiple_chunks/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/003_multiple_chunks/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/004_move_to_new_directory/expected/old/other.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/004_move_to_new_directory/expected/old/other.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/004_move_to_new_directory/expected/renamed/dir/name.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/004_move_to_new_directory/expected/renamed/dir/name.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/004_move_to_new_directory/input/old/name.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/004_move_to_new_directory/input/old/name.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/004_move_to_new_directory/input/old/other.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/004_move_to_new_directory/input/old/other.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/004_move_to_new_directory/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/004_move_to_new_directory/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/005_rejects_empty_patch/expected/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/005_rejects_empty_patch/expected/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/005_rejects_empty_patch/input/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/005_rejects_empty_patch/input/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/005_rejects_empty_patch/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/005_rejects_empty_patch/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/006_rejects_missing_context/expected/modify.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/006_rejects_missing_context/expected/modify.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/006_rejects_missing_context/input/modify.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/006_rejects_missing_context/input/modify.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/006_rejects_missing_context/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/006_rejects_missing_context/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/007_rejects_missing_file_delete/expected/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/007_rejects_missing_file_delete/expected/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/007_rejects_missing_file_delete/input/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/007_rejects_missing_file_delete/input/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/007_rejects_missing_file_delete/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/007_rejects_missing_file_delete/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/008_rejects_empty_update_hunk/expected/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/008_rejects_empty_update_hunk/expected/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/008_rejects_empty_update_hunk/input/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/008_rejects_empty_update_hunk/input/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/008_rejects_empty_update_hunk/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/008_rejects_empty_update_hunk/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/009_requires_existing_file_for_update/expected/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/009_requires_existing_file_for_update/expected/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/009_requires_existing_file_for_update/input/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/009_requires_existing_file_for_update/input/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/009_requires_existing_file_for_update/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/009_requires_existing_file_for_update/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/010_move_overwrites_existing_destination/expected/old/other.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/010_move_overwrites_existing_destination/expected/old/other.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/010_move_overwrites_existing_destination/expected/renamed/dir/name.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/010_move_overwrites_existing_destination/expected/renamed/dir/name.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/010_move_overwrites_existing_destination/input/old/name.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/010_move_overwrites_existing_destination/input/old/name.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/010_move_overwrites_existing_destination/input/old/other.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/010_move_overwrites_existing_destination/input/old/other.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/010_move_overwrites_existing_destination/input/renamed/dir/name.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/010_move_overwrites_existing_destination/input/renamed/dir/name.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/010_move_overwrites_existing_destination/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/010_move_overwrites_existing_destination/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/011_add_overwrites_existing_file/expected/duplicate.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/011_add_overwrites_existing_file/expected/duplicate.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/011_add_overwrites_existing_file/input/duplicate.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/011_add_overwrites_existing_file/input/duplicate.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/011_add_overwrites_existing_file/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/011_add_overwrites_existing_file/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/012_delete_directory_fails/expected/dir/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/012_delete_directory_fails/expected/dir/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/012_delete_directory_fails/input/dir/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/012_delete_directory_fails/input/dir/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/012_delete_directory_fails/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/012_delete_directory_fails/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/013_rejects_invalid_hunk_header/expected/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/013_rejects_invalid_hunk_header/expected/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/013_rejects_invalid_hunk_header/input/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/013_rejects_invalid_hunk_header/input/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/013_rejects_invalid_hunk_header/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/013_rejects_invalid_hunk_header/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/014_update_file_appends_trailing_newline/expected/no_newline.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/014_update_file_appends_trailing_newline/expected/no_newline.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/014_update_file_appends_trailing_newline/input/no_newline.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/014_update_file_appends_trailing_newline/input/no_newline.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/014_update_file_appends_trailing_newline/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/014_update_file_appends_trailing_newline/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/015_failure_after_partial_success_leaves_changes/expected/created.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/015_failure_after_partial_success_leaves_changes/expected/created.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/015_failure_after_partial_success_leaves_changes/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/015_failure_after_partial_success_leaves_changes/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/016_pure_addition_update_chunk/expected/input.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/016_pure_addition_update_chunk/expected/input.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/016_pure_addition_update_chunk/input/input.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/016_pure_addition_update_chunk/input/input.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/016_pure_addition_update_chunk/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/016_pure_addition_update_chunk/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/017_whitespace_padded_hunk_header/expected/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/017_whitespace_padded_hunk_header/expected/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/017_whitespace_padded_hunk_header/input/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/017_whitespace_padded_hunk_header/input/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/017_whitespace_padded_hunk_header/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/017_whitespace_padded_hunk_header/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/018_whitespace_padded_patch_markers/expected/file.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/018_whitespace_padded_patch_markers/expected/file.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/018_whitespace_padded_patch_markers/input/file.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/018_whitespace_padded_patch_markers/input/file.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/018_whitespace_padded_patch_markers/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/018_whitespace_padded_patch_markers/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/019_unicode_simple/expected/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/019_unicode_simple/expected/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/019_unicode_simple/input/foo.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/019_unicode_simple/input/foo.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/019_unicode_simple/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/019_unicode_simple/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/020_delete_file_success/expected/keep.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/020_delete_file_success/expected/keep.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/020_delete_file_success/input/keep.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/020_delete_file_success/input/keep.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/020_delete_file_success/input/obsolete.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/020_delete_file_success/input/obsolete.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/020_delete_file_success/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/020_delete_file_success/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/020_whitespace_padded_patch_marker_lines/expected/file.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/020_whitespace_padded_patch_marker_lines/expected/file.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/020_whitespace_padded_patch_marker_lines/input/file.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/020_whitespace_padded_patch_marker_lines/input/file.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/020_whitespace_padded_patch_marker_lines/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/020_whitespace_padded_patch_marker_lines/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/021_update_file_deletion_only/expected/lines.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/021_update_file_deletion_only/expected/lines.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/021_update_file_deletion_only/input/lines.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/021_update_file_deletion_only/input/lines.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/021_update_file_deletion_only/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/021_update_file_deletion_only/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/022_update_file_end_of_file_marker/expected/tail.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/022_update_file_end_of_file_marker/expected/tail.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario expected output state for future compatibility and regression tests.

### `tests/fixtures/scenarios/022_update_file_end_of_file_marker/input/tail.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/022_update_file_end_of_file_marker/input/tail.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario input state for future compatibility and regression tests.

### `tests/fixtures/scenarios/022_update_file_end_of_file_marker/patch.txt`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/022_update_file_end_of_file_marker/patch.txt`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: upstream scenario patch input for future compatibility and regression tests.

### `tests/fixtures/scenarios/README.md`
- Upstream path: `codex/codex-rs/apply-patch/tests/fixtures/scenarios/README.md`
- Upstream revision: `24a2d0c696d8b1b3d0c137f8e67126b33b07d189`
- Local changes: none; copied verbatim into the test fixtures tree.
- Reason: documents the portable scenario fixture layout used by the upstream apply-patch tests.
