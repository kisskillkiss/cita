// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// This software is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This software is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! General error types for use in ethcore.

use basic_types::LogBloom;
use cita_ed25519::Error as EthkeyError;


use header::BlockNumber;
use std::fmt;
use util::*;
use cita_types::{H256, U256, U512};
use snapshot::error::Error as SnapshotError;

#[derive(Debug, PartialEq, Clone, Copy)]
/// Errors concerning transaction processing.
pub enum TransactionError {
    /// Transaction is already imported to the queue
    AlreadyImported,
    /// Transaction is not valid anymore (state already has higher nonce)
    Old,
    /// Transaction has too low fee
    /// (there is already a transaction with the same sender-nonce but higher gas price)
    TooCheapToReplace,
    /// Transaction was not imported to the queue because limit has been reached.
    LimitReached,
    /// Transaction's gas price is below threshold.
    InsufficientGasPrice {
        /// Minimal expected gas price
        minimal: U256,
        /// Transaction gas price
        got: U256,
    },
    /// Transaction's gas is below currently set minimal gas requirement.
    InsufficientGas {
        /// Minimal expected gas
        minimal: U256,
        /// Transaction gas
        got: U256,
    },
    /// Sender doesn't have enough funds to pay for this transaction
    InsufficientBalance {
        /// Senders balance
        balance: U256,
        /// Transaction cost
        cost: U256,
    },
    /// Transactions gas is higher then current gas limit
    GasLimitExceeded {
        /// Current gas limit
        limit: U256,
        /// Declared transaction gas
        got: U256,
    },
    /// Transaction's gas limit (aka gas) is invalid.
    InvalidGasLimit(OutOfBounds<U256>),
    /// Transaction sender is banned.
    SenderBanned,
    /// Transaction receipient is banned.
    RecipientBanned,
    /// Contract creation code is banned.
    CodeBanned,
    /// Invalid network ID given.
    InvalidNetworkId,
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TransactionError::*;
        let msg = match *self {
            AlreadyImported => "Already imported".into(),
            Old => "No longer valid".into(),
            TooCheapToReplace => "Gas price too low to replace".into(),
            LimitReached => "Transaction limit reached".into(),
            InsufficientGasPrice { minimal, got } => {
                format!("Insufficient gas price. Min={}, Given={}", minimal, got)
            }
            InsufficientGas { minimal, got } => {
                format!("Insufficient gas. Min={}, Given={}", minimal, got)
            }
            InsufficientBalance { balance, cost } => {
                format!("Insufficient balance for transaction. Balance={}, Cost={}", balance, cost)
            }
            GasLimitExceeded { limit, got } => {
                format!("Gas limit exceeded. Limit={}, Given={}", limit, got)
            }
            InvalidGasLimit(ref err) => format!("Invalid gas limit. {}", err),
            SenderBanned => "Sender is temporarily banned.".into(),
            RecipientBanned => "Recipient is temporarily banned.".into(),
            CodeBanned => "Contract code is temporarily banned.".into(),
            InvalidNetworkId => {
                "Transaction of this network ID is not allowed on this chain.".into()
            }
        };

        f.write_fmt(format_args!("Transaction error ({})", msg))
    }
}

#[allow(unknown_lints, large_enum_variant)] // TODO clippy
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
/// Errors concerning block processing.
pub enum BlockError {
    /// Extra data is of an invalid length.
    ExtraDataOutOfBounds(OutOfBounds<usize>),
    /// Seal is incorrect format.
    InvalidSealArity(Mismatch<usize>),
    /// Block has too much gas used.
    TooMuchGasUsed(OutOfBounds<U256>),
    /// State root header field is invalid.
    InvalidStateRoot(Mismatch<H256>),
    /// Gas used header field is invalid.
    InvalidGasUsed(Mismatch<U256>),
    /// Transactions root header field is invalid.
    InvalidTransactionsRoot(Mismatch<H256>),
    /// Difficulty is out of range; this can be used as an looser error prior to getting a definitive
    /// value for difficulty. This error needs only provide bounds of which it is out.
    DifficultyOutOfBounds(OutOfBounds<U256>),
    /// Difficulty header field is invalid; this is a strong error used after getting a definitive
    /// value for difficulty (which is provided).
    InvalidDifficulty(Mismatch<U256>),
    /// Seal element of type H256 (max_hash for Ethash, but could be something else for
    /// other seal engines) is out of bounds.
    MismatchedH256SealElement(Mismatch<H256>),
    /// Proof-of-work aspect of seal, which we assume is a 256-bit value, is invalid.
    InvalidProofOfWork(OutOfBounds<U256>),
    /// Some low-level aspect of the seal is incorrect.
    InvalidSeal,
    /// Gas limit header field is invalid.
    InvalidGasLimit(OutOfBounds<U256>),
    /// Receipts trie root header field is invalid.
    InvalidReceiptsRoot(Mismatch<H256>),
    /// Timestamp header field is invalid.
    InvalidTimestamp(OutOfBounds<u64>),
    /// Log bloom header field is invalid.
    InvalidLogBloom(Mismatch<LogBloom>),
    /// Parent hash field of header is invalid; this is an invalid error indicating a logic flaw in the codebase.
    /// TODO: remove and favour an assert!/panic!.
    InvalidParentHash(Mismatch<H256>),
    /// Number field of header is invalid.
    InvalidNumber(Mismatch<BlockNumber>),
    /// Block number isn't sensible.
    RidiculousNumber(OutOfBounds<BlockNumber>),
    /// Parent given is unknown.
    UnknownParent(H256),
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BlockError::*;

        let msg = match *self {
            ExtraDataOutOfBounds(ref oob) => format!("Extra block data too long. {}", oob),
            InvalidSealArity(ref mis) => format!("Block seal in incorrect format: {}", mis),
            TooMuchGasUsed(ref oob) => format!("Block has too much gas used. {}", oob),
            InvalidStateRoot(ref mis) => format!("Invalid state root in header: {}", mis),
            InvalidGasUsed(ref mis) => format!("Invalid gas used in header: {}", mis),
            InvalidTransactionsRoot(ref mis) => {
                format!("Invalid transactions root in header: {}", mis)
            }
            DifficultyOutOfBounds(ref oob) => format!("Invalid block difficulty: {}", oob),
            InvalidDifficulty(ref mis) => format!("Invalid block difficulty: {}", mis),
            MismatchedH256SealElement(ref mis) => format!("Seal element out of bounds: {}", mis),
            InvalidProofOfWork(ref oob) => format!("Block has invalid PoW: {}", oob),
            InvalidSeal => "Block has invalid seal.".into(),
            InvalidGasLimit(ref oob) => format!("Invalid gas limit: {}", oob),
            InvalidReceiptsRoot(ref mis) => {
                format!("Invalid receipts trie root in header: {}", mis)
            }
            InvalidTimestamp(ref oob) => format!("Invalid timestamp in header: {}", oob),
            InvalidLogBloom(ref oob) => format!("Invalid log bloom in header: {}", oob),
            InvalidParentHash(ref mis) => format!("Invalid parent hash: {}", mis),
            InvalidNumber(ref mis) => format!("Invalid number in header: {}", mis),
            RidiculousNumber(ref oob) => format!("Implausible block number. {}", oob),
            UnknownParent(ref hash) => format!("Unknown parent: {}", hash),
        };

        f.write_fmt(format_args!("Block error ({})", msg))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Import to the block queue result
pub enum ImportError {
    /// Already in the block chain.
    AlreadyInChain,
    /// Already in the block queue.
    AlreadyQueued,
    /// Already marked as bad from a previous import (could mean parent is bad).
    KnownBad,
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            ImportError::AlreadyInChain => "block already in chain",
            ImportError::AlreadyQueued => "block already in the block queue",
            ImportError::KnownBad => "block known to be bad",
        };

        f.write_fmt(format_args!("Block import error ({})", msg))
    }
}

/// Result of executing the transaction.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "ipc", binary)]
pub enum ExecutionError {
    /// Returned when there gas paid for transaction execution is
    /// lower than base gas required.
    NotEnoughBaseGas {
        /// Absolute minimum gas required.
        required: U256,
        /// Gas provided.
        got: U256,
    },
    /// Returned when block (gas_used + gas) > gas_limit.
    ///
    /// If gas =< gas_limit, upstream may try to execute the transaction
    /// in next block.
    BlockGasLimitReached {
        /// Gas limit of block for transaction.
        gas_limit: U256,
        /// Gas used in block prior to transaction.
        gas_used: U256,
        /// Amount of gas in block.
        gas: U256,
    },
    AccountGasLimitReached {
        /// Account Gas limit left
        gas_limit: U256,
        /// Amount of gas in transaction
        gas: U256,
    },
    /// Returned when transaction nonce does not match state nonce.
    InvalidNonce {
        /// Nonce expected.
        expected: U256,
        /// Nonce found.
        got: U256,
    },
    /// Returned when cost of transaction (value + gas_price * gas) exceeds
    /// current sender balance.
    NotEnoughCash {
        /// Minimum required balance.
        required: U512,
        /// Actual balance.
        got: U512,
    },
    NoTransactionPermission,
    NoContractPermission,
    NoCallPermission,
    /// Returned when internal evm error occurs.
    ExecutionInternal(String),
    /// Returned when generic transaction occurs
    TransactionMalformed(String),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ExecutionError::*;

        let msg = match *self {
            NotEnoughBaseGas { ref required, ref got } => format!("Not enough base quota. {} is required, but only {} paid", required, got),
            BlockGasLimitReached {
                ref gas_limit,
                ref gas_used,
                ref gas,
            } => format!("Block gas limit reached. The limit is {}, {} has already been used, and {} more is required", gas_limit, gas_used, gas),
            AccountGasLimitReached { ref gas_limit, ref gas } => format!("Account gas limit reached. The limit is {}, {} more is required", gas_limit, gas),
            InvalidNonce { ref expected, ref got } => format!("Invalid transaction nonce: expected {}, found {}", expected, got),
            NotEnoughCash { ref required, ref got } => format!("Cost of transaction exceeds sender balance. {} is required but the sender only has {}", required, got),
            ExecutionInternal(ref msg) => msg.clone(),
            TransactionMalformed(ref err) => format!("Malformed transaction: {}", err),
            NoTransactionPermission => "No transaction permission".to_owned(),
            NoContractPermission => "No contract permission".to_owned(),
            NoCallPermission => "No call contract permission".to_owned(),
        };

        f.write_fmt(format_args!("Transaction execution error ({}).", msg))
    }
}

#[allow(unknown_lints, large_enum_variant)] // TODO clippy
#[derive(Debug)]
/// General error type which should be capable of representing all errors in ethcore.
pub enum Error {
    /// Error concerning a utility.
    Util(UtilError),
    /// Error concerning block processing.
    Block(BlockError),
    /// Unknown engine given.
    UnknownEngineName(String),
    /// Error concerning EVM code execution.
    Execution(ExecutionError),
    /// Error concerning transaction processing.
    Transaction(TransactionError),
    /// Error concerning block import.
    Import(ImportError),
    /// PoW hash is invalid or out of date.
    PowHashInvalid,
    /// The value of the nonce or mishash is invalid.
    PowInvalid,
    /// Error concerning TrieDBs
    Trie(TrieError),
    /// Standard io error.
    StdIo(::std::io::Error),
    /// Snappy error.
    Snappy(::util::snappy::SnappyError),
    /// Ethkey error.
    Ethkey(EthkeyError),
    /// Snapshot error
    Snapshot(SnapshotError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Util(ref err) => err.fmt(f),
            Error::Block(ref err) => err.fmt(f),
            Error::Execution(ref err) => err.fmt(f),
            Error::Transaction(ref err) => err.fmt(f),
            Error::Import(ref err) => err.fmt(f),
            Error::UnknownEngineName(ref name) => {
                f.write_fmt(format_args!("Unknown engine name ({})", name))
            }
            Error::PowHashInvalid => f.write_str("Invalid or out of date PoW hash."),
            Error::PowInvalid => f.write_str("Invalid nonce or mishash"),
            Error::Trie(ref err) => err.fmt(f),
            Error::StdIo(ref err) => err.fmt(f),
            Error::Snappy(ref err) => err.fmt(f),
            Error::Ethkey(ref err) => err.fmt(f),
            Error::Snapshot(ref err) => err.fmt(f),
        }
    }
}

/// Result of import block operation.
pub type ImportResult = Result<H256, Error>;

impl From<TransactionError> for Error {
    fn from(err: TransactionError) -> Error {
        Error::Transaction(err)
    }
}

impl From<ImportError> for Error {
    fn from(err: ImportError) -> Error {
        Error::Import(err)
    }
}

impl From<BlockError> for Error {
    fn from(err: BlockError) -> Error {
        Error::Block(err)
    }
}

impl From<ExecutionError> for Error {
    fn from(err: ExecutionError) -> Error {
        Error::Execution(err)
    }
}

impl From<::rlp::DecoderError> for Error {
    fn from(err: ::rlp::DecoderError) -> Error {
        Error::Util(UtilError::Decoder(err))
    }
}

impl From<UtilError> for Error {
    fn from(err: UtilError) -> Error {
        Error::Util(err)
    }
}

impl From<TrieError> for Error {
    fn from(err: TrieError) -> Error {
        Error::Trie(err)
    }
}

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error::StdIo(err)
    }
}

impl From<snappy::SnappyError> for Error {
    fn from(err: snappy::SnappyError) -> Error {
        Error::Snappy(err)
    }
}

impl From<EthkeyError> for Error {
    fn from(err: EthkeyError) -> Error {
        Error::Ethkey(err)
    }
}

impl From<SnapshotError> for Error {
    fn from(err: SnapshotError) -> Error {
        match err {
            SnapshotError::Trie(err) => err.into(),
            SnapshotError::Decoder(err) => err.into(),
            other => Error::Snapshot(other),
        }
    }
}

// TODO: uncomment below once https://github.com/rust-lang/rust/issues/27336 sorted.
/*#![feature(concat_idents)]
macro_rules! assimilate {
    ($name:ident) => (
        impl From<concat_idents!($name, Error)> for Error {
            fn from(err: concat_idents!($name, Error)) -> Error {
                Error:: $name (err)
            }
        }
    )
}
assimilate!(FromHex);
assimilate!(BaseData);*/
