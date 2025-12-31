# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-12-31

### Added

#### Rust Layer
- **Input Processing**: Plain text, Wikipedia XML, and HTML readers with streaming support
- **Text Processing**: Unicode normalization, language-aware pre-tokenization (EN, DE, ZH, JP, KR)
- **BPE Tokenizer**: Training, encoding, decoding with binary serialization
- **CLI Commands**: `train`, `similar`, `analogy`, `embed`, `bpe-train`, `bpe-encode`, `info`
- **FFI Bridge**: Type-safe Rust-Fortran bindings with error handling

#### Fortran Layer
- **Word2Vec Core**: Skip-gram with negative sampling, BLAS-accelerated (sdot, saxpy)
- **Parallel Training**: OpenMP with Hogwild-style updates
- **Checkpointing**: Binary save/load of training state
- **Thermal Monitoring**: CPU temperature monitoring via sysfs
- **Graceful Shutdown**: Interrupt handling with state preservation

#### Documentation
- Comprehensive README with Mermaid diagrams
- Translations: German, French, Traditional Chinese, Korean, Japanese
- File format specifications (BPE vocab, model checkpoint)

### Technical Highlights
- Zero external Rust dependencies (stdlib only)
- 162 tests passing
- Dual-language architecture: Rust for text, Fortran for numerics

[1.0.0]: https://github.com/hwang-fu/wvec/releases/tag/v1.0.0
