use crate::{
	Bytes32, Descriptor, EnumMatter, Matter, MatterForm, PermMatter, Result, StateReader, Vec,
	H256, OID,
};
use sp_std::collections::btree_map::BTreeMap;
use thiserror::Error;

macro_rules! ensure {
	($cond:expr, $err:expr) => {
		if !$cond {
			return Err($err);
		}
	};
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ElementError {
	#[error("invalid element source")]
	InvalidElementSource,
	#[error("invalid picker byte sequence")]
	InvalidPickerPadding,
	#[error("not a collection matter")]
	NotCollection,
	#[error("failed to load enum matter")]
	EnumMatterFrom,
	#[error("failed to read enum row")]
	EnumMatterRowAt,
	#[error("failed to load perm matter")]
	PermMatterFrom,
	#[error("failed to read perm row")]
	PermMatterRowAt,
	#[error("missing here collection")]
	NoHereCollection,
	#[error("missing custom picker")]
	NoCustomPicker,
	#[error("previous revision does not exist")]
	NoPreviousRevision,
	#[error("state access error")]
	StateReaderGetMatter,
	#[error("state access error")]
	StateReaderGetSnapshot,
	#[error("failed to get cached row")]
	CacheGet,
	#[error("out of row bounds")]
	RowOutOfBounds,
	#[error("out of col bounds")]
	ColOutOfBounds,
	#[error("invalid mut bits")]
	InvalidMutBits,
	#[error("result length mismatch")]
	ResultLengthMismatch,
	#[error("invalid element count")]
	InvalidElementLength,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PickFrom {
	HereElements = 0,
	HereCollection = 1,
	SetData = 2,
	KindData = 4,
	ObjectData = 8,
}

impl Default for PickFrom {
	fn default() -> Self {
		PickFrom::HereElements
	}
}

impl PickFrom {
	#[inline]
	pub fn from_nibble(n: u8) -> Result<Self, ElementError> {
		use PickFrom::*;
		Ok(match n {
			0 => HereElements,
			1 => HereCollection,
			2 => SetData,
			4 => KindData,
			8 => ObjectData,
			_ => return Err(ElementError::InvalidElementSource),
		})
	}
}

#[derive(Clone, Copy, Default)]
pub struct PickerFlags {
	mut_bits: u16,
	custom: bool,
	here_coll: bool,
	row_from: PickFrom,
}

impl PickerFlags {
	pub fn new() -> Self {
		PickerFlags::default()
	}

	pub fn with_here_coll(self) -> Self {
		Self { here_coll: true, row_from: PickFrom::HereCollection, ..self }
	}

	pub fn with_row_from(self, row_from: PickFrom) -> Self {
		let here_coll = if row_from == PickFrom::HereCollection { true } else { false };
		Self { here_coll, row_from, ..self }
	}

	pub fn with_picker(self) -> Self {
		Self { custom: true, ..self }
	}

	pub fn decode(v: u32) -> Result<Self, ElementError> {
		let row_from = PickFrom::from_nibble((v & 0x0F) as u8)?;
		let here_coll = if row_from == PickFrom::HereCollection { true } else { false };
		Ok(Self {
			mut_bits: ((v >> 16) & 0xFFFF) as u16,
			custom: (v & 0b0001_0000) != 0,
			here_coll,
			row_from,
		})
	}

	pub fn encode(&self) -> u32 {
		((self.mut_bits as u32) << 16) | (u32::from(self.custom) << 4) | (self.row_from as u32)
	}
}

#[derive(Clone, Copy)]
pub struct PickOne {
	src: PickFrom,
	idx: u8,
}

impl PickOne {
	pub fn decode(byte: u8) -> Result<Self, ElementError> {
		Ok(Self { src: PickFrom::from_nibble(byte >> 4)?, idx: byte & 0x0F })
	}

	pub fn encode(&self) -> u8 {
		(self.src as u8) << 4 | (self.idx & 0x0F)
	}

	pub fn decode2(byte: u8) -> Result<Self, ElementError> {
		Ok(Self { src: PickFrom::from_nibble((!byte) >> 4)?, idx: byte & 0x0F })
	}

	pub fn encode2(&self) -> u8 {
		(!(self.src as u8)) << 4 | (self.idx & 0x0F)
	}
}

#[derive(Clone)]
pub struct PickMany {
	picks: Vec<PickOne>,
}

impl PickMany {
	/// First 16 bytes only; stop at first zero and require remaining to be zero.
	pub fn decode(picker: &Bytes32) -> Result<Self, ElementError> {
		let mut picks = Vec::new();
		let mut padded = false;
		for &b in picker.iter().take(16) {
			if b == 0 {
				padded = true;
				continue;
			}
			if padded {
				return Err(ElementError::InvalidPickerPadding);
			}
			picks.push(PickOne::decode2(b)?);
		}
		Ok(Self { picks })
	}

	pub fn encode(&self) -> Bytes32 {
		let mut out = [0u8; 32];
		for (i, pick) in self.picks.iter().take(16).enumerate() {
			out[i] = pick.encode2();
		}
		out
	}
}

pub struct ElementPicker {
	flags: PickerFlags,
	here_elems: Vec<Bytes32>,
	here_coll: Option<Bytes32>,
	custom: Option<PickMany>,
}

impl ElementPicker {
	pub fn new(flags: u32, mut elems: Vec<Bytes32>) -> Result<Self, ElementError> {
		let flags = PickerFlags::decode(flags)?;
		// elems = [here_elems] [here_coll?] [custom_picker?]
		let custom = if flags.custom {
			let picker = elems.pop().ok_or(ElementError::NoCustomPicker)?;
			Some(PickMany::decode(&picker)?)
		} else {
			None
		};

		let here_coll = if flags.here_coll {
			Some(elems.pop().ok_or(ElementError::NoHereCollection)?)
		} else {
			None
		};

		let here_elems = elems;
		Ok(Self { flags, here_elems, here_coll, custom })
	}

	pub fn resolve<E, S: StateReader<E>>(
		&self,
		state: &mut S,
		oid: &OID,
		desc: &Descriptor,
	) -> Result<Vec<Bytes32>, ElementError> {
		let row_index = oid.id.saturating_sub(1);
		if self.custom.is_none() {
			return self.pick_row(state, oid, desc, self.flags.row_from, row_index);
		} else {
			let picker = self.custom.as_ref().unwrap();
			let mut cache: BTreeMap<PickFrom, Vec<Bytes32>> = BTreeMap::new();
			let mut elems = Vec::with_capacity(picker.picks.len());
			for p in picker.picks.iter() {
				let row = self.pick_row_cached(state, oid, desc, p.src, row_index, &mut cache)?;
				let elem = *row.get(p.idx as usize).ok_or(ElementError::ColOutOfBounds)?;
				elems.push(elem);
			}
			Ok(elems)
		}
	}

	pub fn patch(
		mut prev: Vec<Bytes32>,
		resolved: Vec<Bytes32>,
		flags_mut_bits: u16,
	) -> Result<Vec<Bytes32>, ElementError> {
		let n = prev.len();
		ensure!(n <= 16, ElementError::InvalidElementLength);
		let mask = !0u16 << (16 - n); // highest n bits

		let mut_bits = flags_mut_bits & mask;
		ensure!(mut_bits == flags_mut_bits, ElementError::InvalidMutBits);

		// Full replace.
		if mut_bits == 0 {
			ensure!(resolved.len() == n, ElementError::ResultLengthMismatch);
			return Ok(resolved);
		}

		// Partial replace.
		ensure!(
			resolved.len() == mut_bits.count_ones() as usize,
			ElementError::ResultLengthMismatch
		);
		let mut j = 0;
		for i in 0..n {
			if (mut_bits & (1u16 << (15 - i))) != 0 {
				prev[i] = resolved[j];
				j += 1;
			}
		}
		Ok(prev)
	}

	fn pick_row_cached<'cache, E, S: StateReader<E>>(
		&self,
		state: &mut S,
		oid: &OID,
		desc: &Descriptor,
		src: PickFrom,
		row: u64,
		cache: &'cache mut BTreeMap<PickFrom, Vec<Bytes32>>,
	) -> Result<&'cache Vec<Bytes32>, ElementError> {
		if !cache.contains_key(&src) {
			let row_data = self.pick_row(state, oid, desc, src, row)?;
			cache.insert(src, row_data);
		}
		cache.get(&src).ok_or_else(|| ElementError::CacheGet)
	}

	fn pick_row<E, S: StateReader<E>>(
		&self,
		state: &mut S,
		oid: &OID,
		desc: &Descriptor,
		src: PickFrom,
		row: u64,
	) -> Result<Vec<Bytes32>, ElementError> {
		use PickFrom::*;
		match src {
			HereElements => Ok(self.here_elems.clone()),
			HereCollection => {
				let hash = self.here_coll.as_ref().ok_or(ElementError::NoHereCollection)?;
				self.pick_coll_row(state, hash, row)
			},
			SetData => {
				let (_, elems) = state
					.get_snapshot(&oid.set_oid(), desc.srev)
					.map_err(|_| ElementError::StateReaderGetSnapshot)?;
				self.pick_coll_row(state, &elems[1], row)
			},
			KindData => {
				let (_, elems) = state
					.get_snapshot(&oid.kind_oid(desc.kind), desc.krev)
					.map_err(|_| ElementError::StateReaderGetSnapshot)?;
				self.pick_coll_row(state, &elems[1], row)
			},
			ObjectData => {
				if desc.rev <= 1 {
					return Err(ElementError::NoPreviousRevision);
				}
				let (_, prev_elems) = state
					.get_snapshot(oid, desc.rev - 1)
					.map_err(|_| ElementError::StateReaderGetSnapshot)?;
				Ok(prev_elems)
			},
		}
	}

	fn pick_coll_row<E, S: StateReader<E>>(
		&self,
		state: &mut S,
		hash: &H256,
		row: u64,
	) -> Result<Vec<Bytes32>, ElementError> {
		let matter = state.get_matter(hash).map_err(|_| ElementError::StateReaderGetMatter)?;
		let coll = CollectionMatter::from_matter(&matter)?;
		coll.row_at(row)
	}
}

pub enum CollectionMatter {
	Enum(EnumMatter),
	Perm(PermMatter),
}

impl CollectionMatter {
	pub fn from_matter(matter: &Matter) -> Result<Self, ElementError> {
		match matter.form {
			x if x == MatterForm::Enum as u8 => EnumMatter::from(&matter.blob)
				.map_err(|_| ElementError::EnumMatterFrom)
				.map(Self::Enum),
			x if x == MatterForm::Perm as u8 => PermMatter::from(&matter.blob)
				.map_err(|_| ElementError::PermMatterFrom)
				.map(Self::Perm),
			_ => Err(ElementError::NotCollection),
		}
	}

	pub fn row_at(&self, row: u64) -> Result<Vec<Bytes32>, ElementError> {
		let Ok(row) = usize::try_from(row) else {
			return Err(ElementError::RowOutOfBounds);
		};
		match self {
			CollectionMatter::Enum(m) => m
				.row_at(row)
				.map_err(|_| ElementError::EnumMatterRowAt)
				.map(|v| v.into_iter().copied().collect()),
			CollectionMatter::Perm(m) => m
				.row_at(row)
				.map_err(|_| ElementError::PermMatterRowAt)
				.map(|v| v.into_iter().copied().collect()),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		to_mime, Arc, Bytes32, Descriptor, ElementPicker, EnumMatter, Facet, Matter, MatterForm,
		PermMatter, PickFrom, PickerFlags, StateReader, Unique, Value, Vec, H256, OID,
	};
	use anyhow::Result;
	use mockall::mock;
	pub struct E;

	mock! {
		pub MyState {}

		impl StateReader<E> for MyState {
			fn get_matter(&mut self, hash: &H256) -> Result<Matter, E>;
			fn get_value(&mut self, tid: &OID, rev: u32) -> Result<Value, E>;
			fn get_unique(&mut self, tid: &OID, rev: u32) -> Result<Unique, E>;
			// objects
			fn get_descriptor(&mut self, oid: &OID, rev: u32) -> Result<Descriptor, E>;
			fn get_snapshot(&mut self, oid: &OID, rev: u32) -> Result<(Descriptor, Vec<Bytes32>), E>;
			fn get_tails(&mut self, oid: &OID, rev: u32) -> Result<Vec<Arc>, E>;
			fn get_facets(&mut self, oid: &OID, rev: u32) -> Result<Vec<Facet>, E>;
			fn get_facet(&mut self, oid: &OID, rev: u32, sel: u32) -> Result<Matter, E>;

			// helpers
			fn get_kind_contract(&mut self, oid: &OID, rev: u32) -> Result<Matter, E>;
		}
	}

	#[test]
	fn resolve_from_coll() -> Result<()> {
		let flags = PickerFlags {
			mut_bits: 0,
			here_coll: true,
			custom: false,
			row_from: PickFrom::HereCollection,
		};
		let encoded = flags.encode();
		println!("encoded = {}", encoded);

		use hex::FromHex;
		let coll =
			Bytes32::from_hex("31422881b15f078500f6012a56e41c2564c266d5bbc59e7638cc4e5864211481")?;

		let coll_content = Vec::<u8>::from_hex("454e554d10020200000000000000000002ff0000000000000000000000000000cb8dd44f076c2a2bc61da2fe9bd5be9201357571a98fcea73737779070cafa780000000000000000000000000000000000000000000000000000000000123456561592b3c5d66e46c470f2b9ac36a855c4d91531239d61f0ec3e571ca51059e80000000000000000000000000000000000000000000000000000000000123456")?;
		let mut elems = vec![coll.clone()];

		let resolver = ElementPicker::new(encoded, elems)?;
		let oid = OID { universe: 31337, set: 17, id: 1 };
		let desc = Descriptor { traits: 0, rev: 1, krev: 1, srev: 1, kind: 17, trev: 0, fsum: 0 };
		let mut state = MockMyState::new();
		let m =
			Matter { form: 208, mime: to_mime(b"application/vnd.every.enum"), blob: coll_content };
		state
			.expect_get_matter()
			.withf(move |h| h == &coll)
			.returning(move |_| Ok(m.clone()));

		let out = resolver.resolve::<E, _>(&mut state, &oid, &desc)?;
		let expected = vec![
			Bytes32::from_hex("cb8dd44f076c2a2bc61da2fe9bd5be9201357571a98fcea73737779070cafa78")?,
			Bytes32::from_hex("0000000000000000000000000000000000000000000000000000000000123456")?,
		];
		assert_eq!(out, expected);
		Ok(())
	}
}
