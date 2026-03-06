# `libaipatch` Patch Format Notes

This document summarizes the `codex` patch format accepted by `libaipatch` v1.

## Envelope

Every patch must be wrapped in the standard markers:

```text
*** Begin Patch
...
*** End Patch
```

The parser is intentionally a little lenient around whitespace and can also
accept heredoc-like wrappers in compatibility mode, but the canonical input for
the library is plain patch text.

## Top-level operations

The library supports the following hunk headers:

```text
*** Add File: <path>
*** Delete File: <path>
*** Update File: <path>
```

Inside an `Update File` hunk, a rename target may be provided immediately after
the header:

```text
*** Move to: <new_path>
```

All paths are interpreted relative to `root_dir`.

## Add File

`Add File` creates a new text file from lines prefixed with `+`.

Example:

```text
*** Begin Patch
*** Add File: notes/hello.txt
+hello
+world
*** End Patch
```

Resulting file contents:

```text
hello
world
```

The resulting file ends with a trailing newline.

## Delete File

`Delete File` removes an existing file.

Example:

```text
*** Begin Patch
*** Delete File: old/data.txt
*** End Patch
```

Deleting a missing file or a directory is rejected.

## Update File

`Update File` applies one or more chunks to an existing file.

Basic example:

```text
*** Begin Patch
*** Update File: src/app.txt
@@
 line one
-line two
+line two updated
*** End Patch
```

Each update chunk contains line-oriented operations:

- ` ` — context line that must remain present;
- `-` — line to remove;
- `+` — line to add.

The parser also supports `@@ <context>` where the text after `@@ ` is stored as
an additional chunk anchor.

Example with explicit chunk context:

```text
*** Begin Patch
*** Update File: src/app.txt
@@ fn main()
 fn main()
-    old_call();
+    new_call();
*** End Patch
```

## Multiple chunks

A single `Update File` hunk may contain multiple chunks.

Example:

```text
*** Begin Patch
*** Update File: src/app.txt
@@
-alpha
+alpha2
@@
-omega
+omega2
*** End Patch
```

Chunks are applied in order against the progressively planned file state.

## Move to

`Move to` is only valid within `Update File` and changes the destination path of
the updated file.

Example:

```text
*** Begin Patch
*** Update File: old/name.txt
*** Move to: renamed/name.txt
@@
-old content
+new content
*** End Patch
```

This updates the file contents and writes the result to the new path.

## End-of-file marker

The format supports an explicit end-of-file marker:

```text
*** End of File
```

This is mainly useful for hunks that need to anchor changes at the end of the
source file.

Example:

```text
*** Begin Patch
*** Update File: src/app.txt
@@
 line one
+line two
*** End of File
*** End Patch
```

## Trailing newline behavior

`libaipatch` follows the current `codex` compatibility behavior of producing
text files with a trailing newline after successful `Add File` and `Update File`
operations.

## Path constraints

The patch format itself is textual, but the engine enforces additional path
rules during validation:

- patch paths must be relative;
- absolute paths are rejected;
- lexical traversal outside `root_dir` is rejected;
- the same rules apply to `Move to` destinations.

## Typical failure modes

A patch is rejected when:

- the begin/end markers are missing;
- a hunk header is malformed;
- an `Add File` hunk has no `+` lines;
- an `Update File` hunk is empty;
- target context cannot be found in the current file;
- a referenced file does not exist for `Update File` or `Delete File`;
- a path violates the library path policy.

## Recommended generation style

For reliable results, patch generators should:

- keep paths stable and relative;
- provide enough context around modified lines;
- avoid unrelated formatting churn;
- split unrelated file edits into separate hunks;
- avoid relying on exact shell invocation syntax as part of the patch payload.
