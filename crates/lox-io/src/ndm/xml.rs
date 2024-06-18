//! The public interface for the XML deserializer type

pub(crate) mod error;

pub use error::XmlDeserializationError;

mod deserializer;

pub use deserializer::FromXmlStr;
