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
	pub fasum: FacetSummary,
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

#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
	feature = "scale",
	derive(Encode, Decode, TypeInfo, DecodeWithMemTracking, MaxEncodedLen)
)]
pub struct FacetSummary {
	/// bit0==1 means "executed"; MSB==1 means "had error"
	rendered: u8,
	/// number of expected facets, clamped to [0, MAX_FACETS]
	total: u8,
	/// bit i (0..=15) indicates error at facet i; we store facet 0 in MSB (bit 15)
	errors: u16,
}

impl FacetSummary {
	pub const MAX_FACETS: usize = 16;
	pub const FLAG_EXECUTED: u8 = 0b0000_0001;
	pub const FLAG_ERROR: u8 = 0b1000_0000;

	#[inline]
	pub const fn new() -> Self {
		Self { rendered: 0, total: 0, errors: 0 }
	}

	#[inline]
	pub fn to_u32(&self) -> u32 {
		((self.rendered as u32) << 24) | ((self.total as u32) << 16) | (self.errors as u32)
	}

	#[inline]
	pub fn from_u32(raw: u32) -> Self {
		Self {
			rendered: ((raw >> 24) & 0xFF) as u8,
			total: ((raw >> 16) & 0xFF) as u8,
			errors: (raw & 0xFFFF) as u16,
		}
	}

	/// Set total facets (clamped). Does not change rendered/errors.
	#[inline]
	pub fn set_total(mut self, total: usize) -> Self {
		self.total = total.min(Self::MAX_FACETS) as u8;
		self
	}

	/// Mark execution result (ok/err). Keeps `total` untouched.
	#[inline]
	pub fn mark_executed_ok(mut self) -> Self {
		self.rendered = Self::FLAG_EXECUTED;
		self
	}
	#[inline]
	pub fn mark_executed_err(mut self) -> Self {
		self.rendered = Self::FLAG_EXECUTED | Self::FLAG_ERROR;
		self
	}

	/// Convenience: set total and execution in one go.
	#[inline]
	pub fn update(mut self, total: usize, error: bool) -> Self {
		self.total = total.min(Self::MAX_FACETS) as u8;
		self.rendered = Self::FLAG_EXECUTED | if error { Self::FLAG_ERROR } else { 0 };
		self
	}

	/// Set/clear error bit for a facet index (0..=15). Out-of-bounds is a no-op.
	#[inline]
	pub fn set_facet_error(mut self, at: usize, error: bool) -> Self {
		if at < Self::MAX_FACETS {
			let bit = 1u16 << (Self::MAX_FACETS as u16 - 1 - at as u16);
			if error {
				self.errors |= bit
			} else {
				self.errors &= !bit
			}
		}
		self
	}

	/// Query error bit; returns false if out-of-bounds.
	#[inline]
	pub fn facet_error(&self, at: usize) -> bool {
		if at >= Self::MAX_FACETS {
			return false;
		}
		let bit = 1u16 << (Self::MAX_FACETS as u16 - 1 - at as u16);
		(self.errors & bit) != 0
	}

	/// Clear all facet errors.
	#[inline]
	pub fn clear_errors(mut self) -> Self {
		self.errors = 0;
		self
	}

	/// How many error bits are set.
	#[inline]
	pub fn error_count(&self) -> u32 {
		self.errors.count_ones()
	}

	/// Indices of facets that errored (0..=15), up to `total`.
	#[inline]
	pub fn error_indices(&self) -> impl Iterator<Item = usize> + '_ {
		(0..Self::MAX_FACETS)
			.filter(move |&i| self.facet_error(i))
			.take(self.total as usize)
	}

	/// Whether execution ran at all.
	#[inline]
	pub fn executed(&self) -> bool {
		(self.rendered & Self::FLAG_EXECUTED) != 0
	}

	/// Whether there was a top-level execution error.
	#[inline]
	pub fn error(&self) -> bool {
		(self.rendered & Self::FLAG_ERROR) != 0
	}

	/// Total facets expected.
	#[inline]
	pub fn total(&self) -> u8 {
		self.total
	}

	/// Are all facets accounted for (executed, with no facet errors)?
	#[inline]
	pub fn complete_ok(&self) -> bool {
		self.executed() && !self.error() && self.errors == 0
	}
}
