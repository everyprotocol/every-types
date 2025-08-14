use crate::Constants;

pub type H256 = [u8; 32];
pub type Bytes32 = [u8; 32];
pub type Bytes16 = [u8; 16];
pub type Bytes = Vec<u8>;

pub type String31 = [u8; 31];
pub type String30 = [u8; 30];

pub type Result<T, E> = sp_std::result::Result<T, E>;
pub type Vec<T> = sp_std::vec::Vec<T>;

#[cfg(feature = "scale")]
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use derive_more::Display;
#[cfg(feature = "scale")]
use scale_info::TypeInfo;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
#[display("{block}:{slot}:{tick}")]
pub struct Time {
    pub block: u64,
    pub slot: u32,
    pub tick: u32,
}

impl From<u128> for Time {
    fn from(value: u128) -> Self {
        let block = (value >> 64) as u64; // upper 64 bits
        let slot = ((value >> 32) & 0xFFFF_FFFF) as u32; // middle 32 bits
        let tick = (value & 0xFFFF_FFFF) as u32; // lower 32 bits
        Self { block, slot, tick }
    }
}

impl From<Time> for u128 {
    fn from(t: Time) -> Self {
        ((t.block as u128) << 64) | ((t.slot as u128) << 32) | (t.tick as u128)
    }
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Default)]
#[display("({block}, {coord})")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct Position {
    pub block: u64,
    pub coord: u64,
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Default)]
#[display("{universe}.{set}.{id}")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct OID {
    pub universe: u64,
    pub set: u64,
    pub id: u64,
}

impl OID {
    pub fn set_oid(&self) -> OID {
        OID {
            universe: self.universe,
            set: Constants::ID_SET_OF_SET,
            id: self.set,
        }
    }

    pub fn kind_oid(&self, kind: u64) -> OID {
        OID {
            universe: self.universe,
            set: Constants::ID_SET_OF_KIND,
            id: kind,
        }
    }

    pub fn of_set(universe: u64, set: u64) -> OID {
        OID {
            universe,
            set: Constants::ID_SET_OF_SET,
            id: set,
        }
    }

    pub fn of_kind(universe: u64, kind: u64) -> OID {
        OID {
            universe: universe,
            set: Constants::ID_SET_OF_KIND,
            id: kind,
        }
    }

    pub fn of_value(universe: u64, value: u64) -> OID {
        OID {
            universe: universe,
            set: Constants::ID_SET_OF_VALUE,
            id: value,
        }
    }

    pub fn of_unique(universe: u64, unique: u64) -> OID {
        OID {
            universe: universe,
            set: Constants::ID_SET_OF_UNIQUE,
            id: unique,
        }
    }
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Default)]
#[display("{set}.{id}")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct SID {
    pub set: u64,
    pub id: u64,
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Default)]
#[display("% kind={kind}, rev={rev}, krev={krev}, srev={srev}, traits={traits:0x}")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct Descriptor {
    pub traits: u32,
    pub rev: u32,
    pub krev: u32,
    pub srev: u32,
    pub kind: u64,
}

#[derive(Debug, Display, PartialEq, Clone)]
#[display("@ {}, form={form}, blob={}B", str_from_fixed_unchecked(&mime), blob.len())]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
pub struct Matter {
    pub form: u8,
    pub mime: String31,
    pub blob: Bytes,
}

#[cfg(feature = "scale")]
impl MaxEncodedLen for Matter {
    fn max_encoded_len() -> usize {
        u8::max_encoded_len()
            .saturating_mul(Constants::MATTER_SPEC_SIZE) // len(form) + len(mime)
            .saturating_add(
                codec::Compact(Constants::MATTER_BLOB_MAX as u32)
                    .encoded_size() // len(blob)
                    .saturating_add(Constants::MATTER_BLOB_MAX),
            )
    }
}

#[derive(Debug, Display, PartialEq, Clone)]
#[display(
    "# {} std={std}, dec={decimals}, code={}, data={}",
    str_from_fixed_unchecked(symbol),
    short_hex(code),
    short_hex(data)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct Unique {
    pub std: u8,
    pub decimals: u8,
    pub symbol: String30,
    pub code: Bytes32,
    pub data: Bytes32,
}

#[derive(Debug, Display, PartialEq, Clone)]
#[display(
    "$ {} std={std}, dec={decimals}, code={}, data={}",
    str_from_fixed_unchecked(symbol),
    short_hex(code),
    short_hex(data)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct Value {
    pub std: u8,
    pub decimals: u8,
    pub symbol: String30,
    pub code: Bytes32,
    pub data: Bytes32,
}

#[derive(Debug, Display, PartialEq, Clone)]
#[display("<- {rel} [{data}] -- [{kind}] {set}.{id}")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct Arc {
    pub kind: u64,
    pub data: u64,
    pub rel: u64,
    pub set: u64,
    pub id: u64,
}

#[derive(Debug, Display, PartialEq, Clone)]
#[display("<> {sel:08x} => 0x{}", short_hex(hash))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct Facet {
    pub sel: u32,
    pub hash: Bytes32,
}

pub fn to_fixed<const N: usize>(input: &[u8]) -> [u8; N] {
    let mut arr = [0u8; N];
    let len = input.len().min(N);
    arr[..len].copy_from_slice(&input[..len]);
    arr
}

pub fn slice_from_fixed<const N: usize>(buf: &[u8; N]) -> &[u8] {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(N);
    &buf[..end]
}

pub fn str_from_fixed<const N: usize>(buf: &[u8; N]) -> Option<&str> {
    sp_std::str::from_utf8(slice_from_fixed(buf)).ok()
}

pub fn str_from_fixed_unchecked<const N: usize>(buf: &[u8; N]) -> &str {
    unsafe { sp_std::str::from_utf8_unchecked(slice_from_fixed(buf)) }
}

pub fn to_mime(input: &[u8]) -> String31 {
    to_fixed(input)
}

pub fn to_symbol(input: &[u8]) -> String30 {
    to_fixed(input)
}

pub struct ShortHex<'a>(pub &'a [u8; 32]);

use core::fmt;
impl<'a> fmt::Display for ShortHex<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let h = self.0;
        write!(
            f,
            "{:02x}{:02x}{:02x}...{:02x}{:02x}{:02x}",
            h[0], h[1], h[2], h[29], h[30], h[31]
        )
    }
}

#[inline]
pub fn short_hex(h: &[u8; 32]) -> ShortHex<'_> {
    ShortHex(h)
}
