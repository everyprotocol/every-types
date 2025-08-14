#![cfg(feature = "storage")]

use crate::state::{MatterKey, MatterValue, ObjectKey, ObjectValue};
use frame_support::{
    pallet_prelude::{OptionQuery, StorageMap},
    traits::StorageInstance,
    Identity,
};

pub struct MatterStoragePrefix;

impl StorageInstance for MatterStoragePrefix {
    #[rustfmt::skip]
    fn pallet_prefix() -> &'static str { "every" }
    const STORAGE_PREFIX: &'static str = "Matter";
}

pub struct ObjectStoragePrefix;

impl StorageInstance for ObjectStoragePrefix {
    #[rustfmt::skip]
    fn pallet_prefix() -> &'static str { "every" }
    const STORAGE_PREFIX: &'static str = "Object";
}

pub type Matter = StorageMap<MatterStoragePrefix, Identity, MatterKey, MatterValue, OptionQuery>;

pub type Object = StorageMap<ObjectStoragePrefix, Identity, ObjectKey, ObjectValue, OptionQuery>;

// pallet_every::Matter::get(&MatterKey::Matter(*hash));
// let mp = MatterPrefix {};
// let mk = MatterKey::Matter(*hash);
// // let aa = MatterStorage {};
// let a = MatterStorage::get(&mk).ok_or(StateError::ValueNotFound)?;
