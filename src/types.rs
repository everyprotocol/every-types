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
#[cfg(feature = "scale")]
use scale_info::TypeInfo;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
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

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "scale",
    derive(Encode, Decode, TypeInfo, DecodeWithMemTracking)
)]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct Token {
    pub std: u8,
    pub decimals: u8,
    pub symbol: String30,
    pub code: Bytes32,
    pub data: Bytes32,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
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
