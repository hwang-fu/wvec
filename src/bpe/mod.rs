//! Byte Pair Encoding (BPE) tokenizer
//!
//! From-scratch implementation for multilingual text.

mod decode;
mod encode;
mod io;
mod train;
mod types;
mod vocab;

pub use decode::decode;
pub use encode::encode;
pub use io::load;
pub use io::save;
pub use train::train;
