use crate::{
	Bytes32, Descriptor, EnumMatter, Matter, MatterForm, PermMatter, Result, StateReader, Vec, OID,
};
use sp_std::collections::btree_map::BTreeMap;
use thiserror::Error;

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
	#[error("out of row bounds")]
	RowOutOfBounds,
	#[error("out of col bounds")]
	ColOutOfBounds,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElementSource {
	Default = 0,
	SetData = 1,
	KindData = 2,
	ObjectData = 3,
	HereCollection = 4,
	HereElements = 5,
}

impl ElementSource {
	#[inline]
	pub fn from_nibble(n: u8) -> Result<Self, ElementError> {
		use ElementSource::*;
		Ok(match n {
			0 => Default,
			1 => SetData,
			2 => KindData,
			3 => ObjectData,
			4 => HereCollection,
			5 => HereElements,
			_ => return Err(ElementError::InvalidElementSource),
		})
	}
}

#[derive(Clone, Copy)]
pub struct ElementFlags {
	mut_bits: u16,
	here_coll: bool,
	custom_picker: bool,
	pick_row_from: ElementSource,
}

impl ElementFlags {
	pub fn decode(v: u32) -> Result<Self, ElementError> {
		let lo8 = (v & 0xFF) as u8;
		Ok(Self {
			mut_bits: ((v >> 16) & 0xFFFF) as u16,
			here_coll: (lo8 & 0b1000_0000) != 0,
			custom_picker: (lo8 & 0b0001_0000) != 0,
			pick_row_from: ElementSource::from_nibble(lo8 & 0x0F)?,
		})
	}

	pub fn encode(&self) -> u32 {
		((self.mut_bits as u32) << 16) |
			(u32::from(self.here_coll) << 7) |
			(u32::from(self.custom_picker) << 4) |
			(self.pick_row_from as u32)
	}
}

#[derive(Clone, Copy)]
pub struct PickOne {
	src: ElementSource,
	idx: u8,
}

impl PickOne {
	pub fn from(byte: u8) -> Result<Self, ElementError> {
		Ok(Self { src: ElementSource::from_nibble(byte >> 4)?, idx: byte & 0x0F })
	}

	/// First 16 bytes only; stop at first zero and require remaining to be zero.
	pub fn parse_picker(picker: &Bytes32) -> Result<Vec<PickOne>, ElementError> {
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
			picks.push(Self::from(b)?);
		}
		Ok(picks)
	}
}

pub struct ElementResolver {
	flags: ElementFlags,
	here_elems: Vec<Bytes32>,
	here_coll: Option<Bytes32>,
	custom_picker: Option<Vec<PickOne>>,
}

impl ElementResolver {
	pub fn new(flags: u32, mut elems: Vec<Bytes32>) -> Result<Self, ElementError> {
		let flags = ElementFlags::decode(flags)?;

		// elems = [here_elems] [(here_collection)?] [(picker)?]
		let custom_picker = if flags.custom_picker {
			let picker = elems.pop().ok_or(ElementError::NoCustomPicker)?;
			Some(PickOne::parse_picker(&picker)?)
		} else {
			None
		};

		let here_coll = if flags.here_coll {
			Some(elems.pop().ok_or(ElementError::NoHereCollection)?)
		} else {
			None
		};

		let here_elems = elems;
		Ok(Self { flags, here_elems, here_coll, custom_picker })
	}

	pub fn resolve<E, S: StateReader<E>>(
		&self,
		state: &mut S,
		oid: &OID,
		desc: &Descriptor,
	) -> Result<Vec<Bytes32>, ElementError> {
		let row_index = oid.id.saturating_sub(1);
		if self.custom_picker.is_none() {
			return self.row_from_source(state, oid, desc, self.flags.pick_row_from, row_index);
		}

		let picker = self.custom_picker.as_ref().unwrap();
		let mut cache: BTreeMap<ElementSource, Vec<Bytes32>> = BTreeMap::new();
		let mut out = Vec::with_capacity(picker.len());
		for p in picker {
			let row = cache.get(&p.src);
			let elem = if let Some(row) = row {
				let elem = *row.get(p.idx as usize).ok_or(ElementError::ColOutOfBounds)?;
				elem
			} else {
				let row = self.row_from_source(state, oid, desc, p.src, row_index)?;
				let elem = *row.get(p.idx as usize).ok_or(ElementError::ColOutOfBounds)?;
				cache.insert(p.src, row);
				elem
			};
			out.push(elem);
		}
		Ok(out)
	}

	fn row_from_source<E, S: StateReader<E>>(
		&self,
		state: &mut S,
		oid: &OID,
		desc: &Descriptor,
		src: ElementSource,
		row: u64,
	) -> Result<Vec<Bytes32>, ElementError> {
		use ElementSource::*;

		match src {
			Default | HereElements => Ok(self.here_elems.clone()),
			HereCollection => {
				let hash = self.here_coll.as_ref().ok_or(ElementError::NoHereCollection)?;
				self.row_from_collection(state, hash, row)
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
			SetData => {
				let (_, elems) = state
					.get_snapshot(&oid.set_oid(), desc.srev)
					.map_err(|_| ElementError::StateReaderGetSnapshot)?;
				self.row_from_collection(state, &elems[1], row)
			},
			KindData => {
				let (_, elems) = state
					.get_snapshot(&oid.kind_oid(desc.kind), desc.krev)
					.map_err(|_| ElementError::StateReaderGetSnapshot)?;
				self.row_from_collection(state, &elems[1], row)
			},
		}
	}

	fn row_from_collection<E, S: StateReader<E>>(
		&self,
		state: &mut S,
		hash: &Bytes32,
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
