use crate::types::{Arc, Bytes32, Descriptor, Facet, Matter, Token, Vec, H256, OID};

pub enum StateError {
    ValueNotFound,
}

pub trait StateReader {
    // elements
    fn get_matter(&self, hash: &H256) -> Result<Matter, StateError>;

    // objects
    fn get_descriptor(&self, oid: &OID, rev: u32) -> Result<Descriptor, StateError>;
    fn get_snapshot(&self, oid: &OID, rev: u32) -> Result<(Descriptor, Vec<Bytes32>), StateError>;
    fn get_tails(&self, oid: &OID, rev: u32) -> Result<Vec<Arc>, StateError>;
    fn get_facets(&self, oid: &OID, rev: u32) -> Result<Vec<Facet>, StateError>;
    fn get_facet(&self, oid: &OID, rev: u32, sel: u32) -> Result<Matter, StateError>;

    // helpers
    fn get_kind_contract(&self, oid: &OID, rev: u32) -> Result<Matter, StateError>;
    fn get_value(&self, universe: u64, index: u64, rev: u32) -> Result<Token, StateError>;
    fn get_unique(&self, universe: u64, index: u64, rev: u32) -> Result<Token, StateError>;
}
