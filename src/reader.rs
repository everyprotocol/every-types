use crate::{
	state::{Arcs, Facets, MatterKey, MatterValue, ObjectKey, ObjectValue, OidRev, Snapshot, Sota},
	storage::{MatterMap, ObjectMap},
	Bytes32, Descriptor, Facet, Matter, StateReader, Unique, Value, H256, OID,
};
use codec::{Decode, Encode};
use derive_more::Display;
use thiserror::Error;

#[derive(Error, Debug, Display)]
pub enum ProviderError {
	ItemNotFound,
	DecodeFailed,
	UnexpectdVariant,
}

#[derive(Error, Debug, Display)]
pub enum StateError {
	ProviderError(ProviderError),
	DataInvalid,
	UnexpectdVariant,
	DecodeFailed,
	MatterNotFound,
	ValueNotFound,
	UniqueNotFound,
	ObjectNotFound,
	SnapshotNotFound,
	TailsNotFound,
	FacetsNotFound,
	FacetSelectorNotFound,
	FacetAssetNotFound,
	FacetAssetInvalid,
}

impl From<ProviderError> for StateError {
	fn from(value: ProviderError) -> Self {
		StateError::ProviderError(value)
	}
}

pub trait StateProvider {
	fn _get(&self, key: &[u8]) -> Option<Vec<u8>>;

	fn _get_sota(&self, oid: &OID) -> Result<Sota, ProviderError> {
		let key = ObjectKey::Sota(OidRev::new(oid, 0));
		let raw = self._get(&ObjectMap::hashed_key_for(key)).ok_or(ProviderError::ItemNotFound)?;
		let val = ObjectValue::decode(&mut &raw[..]).map_err(|_| ProviderError::DecodeFailed)?;
		match val {
			ObjectValue::Sota(sota) => Ok(sota),
			_ => Err(ProviderError::UnexpectdVariant),
		}
	}

	fn _get_snapshot(&self, oid: &OID, rev: u32) -> Result<Snapshot, ProviderError> {
		let key = ObjectKey::Snapshot(OidRev::new(oid, rev));
		let raw = self._get(&ObjectMap::hashed_key_for(key)).ok_or(ProviderError::ItemNotFound)?;
		let val = ObjectValue::decode(&mut &raw[..]).map_err(|_| ProviderError::DecodeFailed)?;
		match val {
			ObjectValue::Snapshot(snap) => Ok(snap),
			_ => Err(ProviderError::UnexpectdVariant),
		}
	}

	fn _get_matter(&self, hash: &H256) -> Result<Matter, ProviderError> {
		let key = MatterKey::Matter(*hash);
		let raw = self._get(&MatterMap::hashed_key_for(key)).ok_or(ProviderError::ItemNotFound)?;
		let val = MatterValue::decode(&mut &raw[..]).map_err(|_| ProviderError::DecodeFailed)?;
		match val {
			MatterValue::Matter(mat) => Ok(mat),
		}
	}

	fn _resolve_rev(&self, oid: &OID, rev0: u32) -> Result<u32, ProviderError> {
		let rev = if rev0 == 0 { self._get_sota(oid)?.desc.rev } else { rev0 };
		Ok(rev)
	}

	fn _resolve_desc(&self, oid: &OID, rev0: u32) -> Result<Descriptor, ProviderError> {
		let desc =
			if rev0 == 0 { self._get_sota(oid)?.desc } else { self._get_snapshot(oid, rev0)?.desc };
		Ok(desc)
	}
}

impl<T> StateReader<StateError> for T
where
	T: StateProvider,
{
	fn get_matter(&self, hash: &H256) -> Result<Matter, StateError> {
		self._get_matter(hash).map_err(StateError::from)
	}

	fn get_value(&self, tid: &OID, rev: u32) -> Result<Value, StateError> {
		let rev = self._resolve_rev(tid, rev)?;
		let snap = self._get_snapshot(tid, rev)?;
		if snap.elems.len() != 3 {
			return Err(StateError::DataInvalid);
		}

		let code = snap.elems[0];
		let data = snap.elems[1];
		let std = snap.elems[2][0];
		let decimals = snap.elems[2][1];
		let mut symbol = [0u8; 30];
		symbol.copy_from_slice(&snap.elems[2][2..32]);
		Ok(Value { std, decimals, symbol, code, data })
	}

	fn get_unique(&self, tid: &OID, rev: u32) -> Result<Unique, StateError> {
		let rev = self._resolve_rev(tid, rev)?;
		let snap = self._get_snapshot(tid, rev)?;
		if snap.elems.len() != 3 {
			return Err(StateError::DataInvalid);
		}

		let code = snap.elems[0];
		let data = snap.elems[1];
		let std = snap.elems[2][0];
		let decimals = snap.elems[2][1];
		let mut symbol = [0u8; 30];
		symbol.copy_from_slice(&snap.elems[2][2..32]);
		Ok(Unique { std, decimals, symbol, code, data })
	}

	fn get_descriptor(&self, oid: &OID, rev: u32) -> Result<Descriptor, StateError> {
		self._resolve_desc(oid, rev).map_err(StateError::from)
	}

	fn get_snapshot(&self, oid: &OID, rev: u32) -> Result<(Descriptor, Vec<Bytes32>), StateError> {
		let rev = self._resolve_rev(oid, rev)?;
		let snap = self._get_snapshot(oid, rev)?;
		Ok((snap.desc, snap.elems))
	}

	fn get_tails(&self, oid: &OID, rev: u32) -> Result<crate::Vec<crate::Arc>, StateError> {
		let rev = self._resolve_rev(oid, rev)?;
		let key = ObjectKey::Tails(OidRev::new(oid, rev));
		let raw = self._get(&ObjectMap::hashed_key_for(key)).ok_or(StateError::TailsNotFound)?;
		let val = ObjectValue::decode(&mut &raw[..]).map_err(|_| StateError::DecodeFailed)?;
		match val {
			ObjectValue::Tails(Arcs { arcs }) => Ok(arcs),
			_ => Err(StateError::DataInvalid),
		}
	}

	fn get_facets(&self, oid: &OID, rev: u32) -> Result<Vec<Facet>, StateError> {
		let rev = self._resolve_rev(oid, rev)?;
		let key = ObjectKey::Facets(OidRev::new(oid, rev));
		let raw = self._get(&ObjectMap::hashed_key_for(key)).ok_or(StateError::FacetsNotFound)?;
		let val = ObjectValue::decode(&mut &raw[..]).map_err(|_| StateError::DecodeFailed)?;
		match val {
			ObjectValue::Facets(Facets { facets }) => Ok(facets),
			_ => Err(StateError::DataInvalid),
		}
	}

	fn get_facet(&self, oid: &OID, rev: u32, sel: u32) -> Result<Matter, StateError> {
		let rev = self._resolve_rev(oid, rev)?;
		let key = ObjectKey::Facets(OidRev::new(oid, rev));
		let raw = self._get(&ObjectMap::hashed_key_for(key)).ok_or(StateError::FacetsNotFound)?;
		let val = ObjectValue::decode(&mut &raw[..]).map_err(|_| StateError::DecodeFailed)?;
		let facets = match val {
			ObjectValue::Facets(Facets { facets }) => facets,
			_ => return Err(StateError::DataInvalid),
		};
		let facet =
			facets.iter().find(|f| f.sel == sel).ok_or(StateError::FacetSelectorNotFound)?;
		self._get_matter(&facet.hash).map_err(StateError::from)
	}

	fn get_kind_contract(&self, oid: &OID, rev: u32) -> Result<Matter, StateError> {
		let desc = self._resolve_desc(oid, rev)?;
		let snap = self._get_snapshot(&oid.kind_oid(desc.kind), desc.krev)?;
		self._get_matter(&snap.elems[0]).map_err(StateError::from)
	}
}
