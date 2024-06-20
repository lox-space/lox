//! The public interface for the `KvnDeserializer` type

mod deserializer;
pub(crate) mod parser;

pub use deserializer::{KvnDeserializer, KvnDeserializerErr};
