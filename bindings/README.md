# Bindings

This directory contains thin language bindings built on top of the public C ABI.

Layout:
- `cpp/` — header-only C++ wrapper, example, and smoke-test;
- `D/` — D module, example, and smoke-test;
- `Makefile` — convenience targets for building the Rust static library and running binding smoke-tests.

Useful targets:

```sh
cd bindings
make cpp-test
make d-test-dmd
make d-test-ldc2
make all
```
