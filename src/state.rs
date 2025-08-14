use crate::{Arc, Bytes32, Constants, Descriptor, Facet, Matter, H256, OID};
type Vec<T> = sp_std::vec::Vec<T>;

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
	pub fsum: FacetSummary,
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

pub type UniverseKey = u64;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct UniverseValue {
	pub initiator: Bytes32,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct HeraldKey {
	pub universe: u64,
	pub herald: Bytes32,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct HeraldValue {}

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

// #[derive(Debug, PartialEq, Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
// pub struct SetSota {
// 	pub owner: Bytes32,
// 	pub updated_at: u128,          // last modified time
// 	pub children_updated_at: u128, // last modified time of children
// 	pub energy: u128,
// }

#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo, DecodeWithMemTracking))]
#[cfg_attr(feature = "scale", derive(MaxEncodedLen))]
pub struct FacetSummary {
	pub facets: u8,   // 0..=16 expected
	pub executed: u8, // true if > 0
	pub errors: u16,  // bitmap of 16 facets
}

impl FacetSummary {
	/// Set/clear error for a given facet index (0..=15).
	pub fn set_error(&mut self, idx: u8, has_error: bool) {
		if idx < 16 {
			let bit = 1u16 << (15 - idx); // MSB→LSB: idx 0 -> bit 15
			if has_error {
				self.errors |= bit;
			} else {
				self.errors &= !bit;
			}
		}
	}

	/// Query if a given facet (0..=15) has an error.
	pub fn has_error(&self, idx: u8) -> bool {
		if idx < 16 {
			let bit = 1u16 << (15 - idx);
			(self.errors & bit) != 0
		} else {
			false
		}
	}
}
