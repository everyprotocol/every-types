use crate::types::{Arc, Bytes32, Descriptor, Facet, Matter, Unique, Value, Vec, H256, OID};

pub trait StateReader<E> {
	// elements
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
