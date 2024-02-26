use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ListingError {
    #[error("Failed access/create file: {0}.")]
    FileError(#[from] io::Error),
    #[error("Failed to deserialize yaml: {0}.")]
    YamlDeserError(#[from] serde_yaml::Error),
    #[error("Failed to deserialize hjson/json: {0}.")]
    HJsonDeserError(#[from] deser_hjson::Error),
    #[error("Conversion error at {bt}: {msg}.")]
    ConversionError { bt: String, msg: String },
}

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("Failed to output: {0}.")]
    OutputError(#[from] std::fmt::Error),
    #[error("Invalid args: {0}.")]
    ArgError(#[from] clap::Error),
    #[error("Generator error {0}.")]
    Error(String),
}
