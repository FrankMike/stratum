//! ## Error Module
//!
//! Defines [`Error`], the central error enum used throughout the Job Declarator Client (JDC).
//!
//! It unifies errors from:
//! - I/O operations
//! - Channels (send/recv)
//! - SV2 stack: Binary, Codec, Noise, Framing, RolesLogic
//! - Locking logic (PoisonError)
//! - Domain-specific issues
//!
//! This module ensures that all errors can be passed around consistently, including across async
//! boundaries.
use ext_config::ConfigError;
use std::fmt;
use stratum_common::roles_logic_sv2::{
    self,
    codec_sv2::{self, binary_sv2, framing_sv2},
    mining_sv2::{ExtendedExtranonce, NewExtendedMiningJob, SetCustomMiningJob},
    parsers_sv2::ParserError,
};

pub type ProxyResult<'a, T> = core::result::Result<T, Error<'a>>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ChannelSendError<'a> {
    SubmitSharesExtended(
        async_channel::SendError<roles_logic_sv2::mining_sv2::SubmitSharesExtended<'a>>,
    ),
    SetNewPrevHash(async_channel::SendError<roles_logic_sv2::mining_sv2::SetNewPrevHash<'a>>),
    NewExtendedMiningJob(async_channel::SendError<NewExtendedMiningJob<'a>>),
    General(String),
    Extranonce(async_channel::SendError<(ExtendedExtranonce, u32)>),
    SetCustomMiningJob(
        async_channel::SendError<roles_logic_sv2::mining_sv2::SetCustomMiningJob<'a>>,
    ),
    NewTemplate(
        async_channel::SendError<(
            roles_logic_sv2::template_distribution_sv2::SetNewPrevHash<'a>,
            Vec<u8>,
        )>,
    ),
}

#[derive(Debug)]
pub enum Error<'a> {
    #[allow(dead_code)]
    VecToSlice32(Vec<u8>),
    /// Errors on bad CLI argument input.
    BadCliArgs,
    /// Errors on bad `config` TOML deserialize.
    BadConfigDeserialize(ConfigError),
    /// Errors from `binary_sv2` crate.
    BinarySv2(binary_sv2::Error),
    /// Errors on bad noise handshake.
    CodecNoise(codec_sv2::noise_sv2::Error),
    /// Errors from `framing_sv2` crate.
    FramingSv2(framing_sv2::Error),
    /// Errors on bad `TcpStream` connection.
    Io(std::io::Error),
    /// Errors on bad `String` to `int` conversion.
    ParseInt(std::num::ParseIntError),
    /// Errors from `roles_logic_sv2` crate.
    RolesSv2Logic(roles_logic_sv2::errors::Error),
    UpstreamIncoming(roles_logic_sv2::errors::Error),
    #[allow(dead_code)]
    SubprotocolMining(String),
    // Locking Errors
    PoisonLock,
    // Channel Receiver Error
    ChannelErrorReceiver(async_channel::RecvError),
    TokioChannelErrorRecv(tokio::sync::broadcast::error::RecvError),
    // Channel Sender Errors
    ChannelErrorSender(ChannelSendError<'a>),
    Infallible(std::convert::Infallible),
    Parser(ParserError),
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            BadCliArgs => write!(f, "Bad CLI arg input"),
            BadConfigDeserialize(ref e) => write!(f, "Bad `config` TOML deserialize: `{e:?}`"),
            BinarySv2(ref e) => write!(f, "Binary SV2 error: `{e:?}`"),
            CodecNoise(ref e) => write!(f, "Noise error: `{e:?}"),
            FramingSv2(ref e) => write!(f, "Framing SV2 error: `{e:?}`"),
            Io(ref e) => write!(f, "I/O error: `{e:?}"),
            ParseInt(ref e) => write!(f, "Bad convert from `String` to `int`: `{e:?}`"),
            RolesSv2Logic(ref e) => write!(f, "Roles SV2 Logic Error: `{e:?}`"),
            SubprotocolMining(ref e) => write!(f, "Subprotocol Mining Error: `{e:?}`"),
            UpstreamIncoming(ref e) => write!(f, "Upstream parse incoming error: `{e:?}`"),
            PoisonLock => write!(f, "Poison Lock error"),
            ChannelErrorReceiver(ref e) => write!(f, "Channel receive error: `{e:?}`"),
            TokioChannelErrorRecv(ref e) => write!(f, "Channel receive error: `{e:?}`"),
            ChannelErrorSender(ref e) => write!(f, "Channel send error: `{e:?}`"),
            VecToSlice32(ref e) => write!(f, "Standard Error: `{e:?}`"),
            Infallible(ref e) => write!(f, "Infallible Error:`{e:?}`"),
            Parser(ref e) => write!(f, "Parser error: `{e:?}`"),
        }
    }
}

impl From<ParserError> for Error<'_> {
    fn from(e: ParserError) -> Self {
        Error::Parser(e)
    }
}

impl From<binary_sv2::Error> for Error<'_> {
    fn from(e: binary_sv2::Error) -> Self {
        Error::BinarySv2(e)
    }
}

impl From<codec_sv2::noise_sv2::Error> for Error<'_> {
    fn from(e: codec_sv2::noise_sv2::Error) -> Self {
        Error::CodecNoise(e)
    }
}

impl From<framing_sv2::Error> for Error<'_> {
    fn from(e: framing_sv2::Error) -> Self {
        Error::FramingSv2(e)
    }
}

impl From<std::io::Error> for Error<'_> {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<std::num::ParseIntError> for Error<'_> {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseInt(e)
    }
}

impl From<roles_logic_sv2::errors::Error> for Error<'_> {
    fn from(e: roles_logic_sv2::errors::Error) -> Self {
        Error::RolesSv2Logic(e)
    }
}

impl From<ConfigError> for Error<'_> {
    fn from(e: ConfigError) -> Self {
        Error::BadConfigDeserialize(e)
    }
}

impl From<async_channel::RecvError> for Error<'_> {
    fn from(e: async_channel::RecvError) -> Self {
        Error::ChannelErrorReceiver(e)
    }
}

impl From<tokio::sync::broadcast::error::RecvError> for Error<'_> {
    fn from(e: tokio::sync::broadcast::error::RecvError) -> Self {
        Error::TokioChannelErrorRecv(e)
    }
}

// *** LOCK ERRORS ***
// impl<'a> From<PoisonError<MutexGuard<'a, proxy::Bridge>>> for Error<'a> {
//     fn from(e: PoisonError<MutexGuard<'a, proxy::Bridge>>) -> Self {
//         Error::PoisonLock(
//             LockError::Bridge(e)
//         )
//     }
// }

// impl<'a> From<PoisonError<MutexGuard<'a, NextMiningNotify>>> for Error<'a> {
//     fn from(e: PoisonError<MutexGuard<'a, NextMiningNotify>>) -> Self {
//         Error::PoisonLock(
//             LockError::NextMiningNotify(e)
//         )
//     }
// }

// *** CHANNEL SENDER ERRORS ***
impl<'a> From<async_channel::SendError<roles_logic_sv2::mining_sv2::SubmitSharesExtended<'a>>>
    for Error<'a>
{
    fn from(
        e: async_channel::SendError<roles_logic_sv2::mining_sv2::SubmitSharesExtended<'a>>,
    ) -> Self {
        Error::ChannelErrorSender(ChannelSendError::SubmitSharesExtended(e))
    }
}

impl<'a> From<async_channel::SendError<roles_logic_sv2::mining_sv2::SetNewPrevHash<'a>>>
    for Error<'a>
{
    fn from(e: async_channel::SendError<roles_logic_sv2::mining_sv2::SetNewPrevHash<'a>>) -> Self {
        Error::ChannelErrorSender(ChannelSendError::SetNewPrevHash(e))
    }
}

impl From<async_channel::SendError<(ExtendedExtranonce, u32)>> for Error<'_> {
    fn from(e: async_channel::SendError<(ExtendedExtranonce, u32)>) -> Self {
        Error::ChannelErrorSender(ChannelSendError::Extranonce(e))
    }
}

impl<'a> From<async_channel::SendError<NewExtendedMiningJob<'a>>> for Error<'a> {
    fn from(e: async_channel::SendError<NewExtendedMiningJob<'a>>) -> Self {
        Error::ChannelErrorSender(ChannelSendError::NewExtendedMiningJob(e))
    }
}

impl<'a> From<async_channel::SendError<SetCustomMiningJob<'a>>> for Error<'a> {
    fn from(e: async_channel::SendError<SetCustomMiningJob<'a>>) -> Self {
        Error::ChannelErrorSender(ChannelSendError::SetCustomMiningJob(e))
    }
}

impl<'a>
    From<
        async_channel::SendError<(
            roles_logic_sv2::template_distribution_sv2::SetNewPrevHash<'a>,
            Vec<u8>,
        )>,
    > for Error<'a>
{
    fn from(
        e: async_channel::SendError<(
            roles_logic_sv2::template_distribution_sv2::SetNewPrevHash<'a>,
            Vec<u8>,
        )>,
    ) -> Self {
        Error::ChannelErrorSender(ChannelSendError::NewTemplate(e))
    }
}

impl From<std::convert::Infallible> for Error<'_> {
    fn from(e: std::convert::Infallible) -> Self {
        Error::Infallible(e)
    }
}
