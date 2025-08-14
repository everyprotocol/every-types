pub struct Constants;

impl Constants {
	// Set IDs
	pub const ID_SET_OF_SET: u64 = 1;
	pub const ID_SET_OF_KIND: u64 = 2;
	pub const ID_SET_OF_REL: u64 = 3;
	pub const ID_SET_OF_VALUE: u64 = 4;
	pub const ID_SET_OF_UNIQUE: u64 = 5;

	// Kind IDs
	pub const ID_KIND_OF_SET: u64 = 1;
	pub const ID_KIND_OF_KIND: u64 = 2;
	pub const ID_KIND_OF_REL: u64 = 3;
	pub const ID_KIND_OF_VALUE: u64 = 4;
	pub const ID_KIND_OF_UNIQUE: u64 = 5;

	// System max IDs
	pub const ID_SET_SYSTEM_MAX: u64 = 16;
	pub const ID_KIND_SYSTEM_MAX: u64 = 16;
	pub const ID_REL_SYSTEM_MAX: u64 = 16;

	// General ID constants
	pub const ID_UNSPECIFIED: u64 = 0;
	pub const ID_MIN: u64 = 1;
	pub const ID_MAX: u64 = u64::MAX - 1;
	pub const ID_WILDCARD: u64 = u64::MAX;

	// Revision markers
	pub const REV_NEW: u32 = 1;
	pub const REV_DESTROYED: u32 = u32::MAX;

	// Capacities
	pub const ELEM_SPEC_CAPACITY: usize = 16;
	pub const REL_SPEC_CAPACITY: usize = 8;
	pub const ADJ_SPEC_CAPACITY: usize = 8;
	pub const TAIL_CAPACITY: usize = 1024;
	pub const FACET_CAPACITY: usize = 16;
	pub const MATTER_SPEC_SIZE: usize = 32;
	pub const MATTER_BLOB_MAX: usize = 1024 * 1024 * 10;
}
