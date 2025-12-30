# wvec

**A from-scratch multilingual Word2Vec implementation in Rust + Modern Fortran**

> Work in Progress

## What This Is

A word embedding system combining:
- **Rust** — Text processing, BPE tokenization, CLI
- **Fortran** — Numerical computation (BLAS, OpenMP)

## Current Progress

| Component | Status | Tests |
|-----------|--------|-------|
| Build System | Done | Cargo + Makefile + FFI |
| Input Readers | Done | text, XML, HTML (57 tests) |
| Text Processing | Done | Normalization, pre-tokenization (38 tests) |
| BPE Tokenizer | Partial | Core types + training done (36 tests) |
| Word2Vec Training | Not started | — |

**Total: 132 tests passing**

## Building

```bash
# Fedora
sudo dnf install gcc-gfortran openblas-devel

# Build & test
cargo build
cargo test
```

## Project Structure

```
wvec/
├── src/
│   ├── input/      # Text, XML, HTML readers
│   ├── text/       # Normalization, pre-tokenization
│   ├── bpe/        # BPE tokenizer
│   ├── cli.rs      # CLI parsing
│   └── ffi.rs      # Fortran bindings
└── fortran/        # Fortran modules (skeleton)
```

## License

MIT
