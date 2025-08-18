use crate::state::{MatterKey, MatterValue, ObjectKey, ObjectValue};
use frame_support::{
	pallet_prelude::{OptionQuery, StorageMap},
	Identity,
};

pub use frame_support::traits::StorageInstance;

pub struct MatterMapPrefix;

impl StorageInstance for MatterMapPrefix {
	#[rustfmt::skip]
	fn pallet_prefix() -> &'static str { "Every" }
	const STORAGE_PREFIX: &'static str = "Matter";
}

pub struct ObjectMapPrefix;

impl StorageInstance for ObjectMapPrefix {
	#[rustfmt::skip]
	fn pallet_prefix() -> &'static str { "Every" }
	const STORAGE_PREFIX: &'static str = "Object";
}

pub type MatterMap = StorageMap<MatterMapPrefix, Identity, MatterKey, MatterValue, OptionQuery>;

pub type ObjectMap = StorageMap<ObjectMapPrefix, Identity, ObjectKey, ObjectValue, OptionQuery>;
