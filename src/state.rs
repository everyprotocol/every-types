use crate::{Arc, Bytes32, Constants, Descriptor, Facet, Matter, Vec, H256, OID};

#[cfg(feature = "scale")]
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
#[cfg(feature = "scale")]
use scale_info::TypeInfo;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct Sota {
	pub desc: Descriptor,
	pub trev: u32,
	pub fasum: u32,
	pub owner: Bytes32,
	pub pos: u128,
	pub mtime: u128,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct OidRev {
	universe: u64,
	set: u64,
	id: u64,
	rev: u32,
}

impl OidRev {
	pub fn new(oid: &OID, rev: u32) -> Self {
		Self { universe: oid.universe, set: oid.set, id: oid.id, rev }
	}
}

#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
pub struct Snapshot {
	pub desc: Descriptor,
	pub trev: u32,
	pub mtime: u128,
	pub elems: Vec<H256>,
}

#[cfg(feature = "scale")]
impl MaxEncodedLen for Snapshot {
	fn max_encoded_len() -> usize {
		H256::max_encoded_len()
			.saturating_mul(Constants::ELEM_SPEC_CAPACITY)
			.saturating_add(codec::Compact(Constants::ELEM_SPEC_CAPACITY as u32).encoded_size())
			.saturating_add(Descriptor::max_encoded_len())
	}
}
#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
pub struct Facets {
	pub facets: Vec<Facet>,
}

#[cfg(feature = "scale")]
impl MaxEncodedLen for Facets {
	fn max_encoded_len() -> usize {
		Facet::max_encoded_len()
			.saturating_mul(Constants::FACET_CAPACITY)
			.saturating_add(codec::Compact(Constants::FACET_CAPACITY as u32).encoded_size())
	}
}

#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
pub struct Arcs {
	pub arcs: Vec<Arc>,
}

#[cfg(feature = "scale")]
impl MaxEncodedLen for Arcs {
	fn max_encoded_len() -> usize {
		Arc::max_encoded_len()
			.saturating_mul(Constants::TAIL_CAPACITY)
			.saturating_add(codec::Compact(Constants::TAIL_CAPACITY as u32).encoded_size())
	}
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub enum ObjectKey {
	Sota(OidRev),
	Snapshot(OidRev),
	Tails(OidRev),
	Facets(OidRev),
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub enum ObjectValue {
	Sota(Sota),
	Snapshot(Snapshot),
	Tails(Arcs),
	Facets(Facets),
}

pub type UniverseId = u64;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct UniverseHerald {
	pub universe: u64,
	pub herald: Bytes32,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct Genesis {
	pub horizon: u128,
	pub otime: u128,
	pub originator: Bytes32,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct Frontier {
	pub furthest: u128,
	pub frontier: u128,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub enum UniverseKey {
	Genesis(UniverseId),
	Frontier(UniverseId),
	Herald(UniverseHerald),
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub enum UniverseValue {
	Genesis(Genesis),
	Frontier(Frontier),
	Herald,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub enum MatterKey {
	Matter(H256),
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub enum MatterValue {
	Matter(Matter),
}
