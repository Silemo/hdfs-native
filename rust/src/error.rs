use std::io;

use prost::DecodeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HdfsError {
    #[error("IO error occurred while communicating with HDFS")]
    IOError(#[from] io::Error),
    #[error("data transfer error")]
    DataTransferError(String),
    #[error("checksums didn't match")]
    ChecksumError,
    #[error("invalid path")]
    InvalidPath(String),
    #[error("invalid argument")]
    InvalidArgument(String),
    #[error("failed to parse URL")]
    UrlParseError(#[from] url::ParseError),
    #[error("file already exists")]
    AlreadyExists(String),
    #[error("operation failed")]
    OperationFailed(String),
    #[error("file not found")]
    FileNotFound(String),
    #[error("blocks not found")]
    BlocksNotFound(String),
    #[error("path is a directory")]
    IsADirectoryError(String),
    #[error("unsupported erasure coding policy")]
    UnsupportedErasureCodingPolicy(String),
    #[error("erasure coding error")]
    ErasureCodingError(String),
    #[error("operation not supported")]
    UnsupportedFeature(String),
    #[error("interal error, this shouldn't happen")]
    InternalError(String),
    #[error("failed to decode RPC response")]
    InvalidRPCResponse(#[from] DecodeError),
    #[error("RPC error")]
    RPCError(String, String),
    #[error("fatal RPC error")]
    FatalRPCError(String, String),
    #[error("SASL error")]
    SASLError(String),
    #[error("GSSAPI error")]
    GSSAPIError(crate::security::gssapi::GssMajorCodes, u32, String),
    #[error("No valid SASL mechanism found")]
    NoSASLMechanism,
    #[error("TLS Error: Client config created incorrectly ")]
    TLSClientConfigError,
    #[error("TLS Error: DNS name invalid")]
    TLSDNSInvalidError,
    #[error("TCP Error: TCP connection failed")]
    TCPConnectionError(String),
}

pub type Result<T> = std::result::Result<T, HdfsError>;
