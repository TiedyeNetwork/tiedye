#![cfg_attr(not(feature = "std"), no_std)]

use primitives::crypto::KeyTypeId;

pub const ORACLE: KeyTypeId = KeyTypeId(*b"orac");

/// Balance of an account.
pub type Balance = u128;

/// An index to a block.
pub type BlockNumber = u32;

