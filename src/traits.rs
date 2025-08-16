use crate::types::{Arc, Bytes32, Descriptor, Facet, Matter, Unique, Value, Vec, H256, OID};

pub trait StateReader<E> {
	// elements
	fn get_matter(&self, hash: &H256) -> Result<Matter, E>;
	fn get_value(&self, tid: &OID, rev: u32) -> Result<Value, E>;
	fn get_unique(&self, tid: &OID, rev: u32) -> Result<Unique, E>;

	// objects
	fn get_descriptor(&self, oid: &OID, rev: u32) -> Result<Descriptor, E>;
	fn get_snapshot(&self, oid: &OID, rev: u32) -> Result<(Descriptor, Vec<Bytes32>), E>;
	fn get_tails(&self, oid: &OID, rev: u32) -> Result<Vec<Arc>, E>;
	fn get_facets(&self, oid: &OID, rev: u32) -> Result<Vec<Facet>, E>;
	fn get_facet(&self, oid: &OID, rev: u32, sel: u32) -> Result<Matter, E>;

	// helpers
	fn get_kind_contract(&self, oid: &OID, rev: u32) -> Result<Matter, E>;
}
