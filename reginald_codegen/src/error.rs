use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed access/create file: {0}.")]
    FileError(#[from] io::Error),
    #[error("Failed to deserialize yaml: {0}.")]
    YamlDeserError(#[from] serde_yaml::Error),
    #[error("Failed to deserialize hjson/json: {0}.")]
    HJsonDeserError(#[from] deser_hjson::Error),
    #[error("Conversion error at {bt}: {msg}.")]
    ConversionError { bt: String, msg: String },
    #[error("Failed to output: {0}.")]
    OutputError(#[from] std::fmt::Error),
    #[error("Generator error: {0}.")]
    GeneratorError(String),
    #[error("Failed to load template: {0}.")]
    TempalteError(#[from] handlebars::TemplateError),
    #[error("Failed to render template: {0}.")]
    TempalteRenderError(#[from] handlebars::RenderError),
    #[error("Validation Error: {0}")]
    VerificationError(String),
}
