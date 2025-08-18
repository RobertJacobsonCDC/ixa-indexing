//! A completely type-erased index.

use std::{
  collections::HashSet,
  hash::Hash
};
use hashbrown::{
  HashTable,
  hash_table::OccupiedEntry
};
use crate::hash128::{one_shot_128};
use crate::EntityId;

type HashValueType = u128;


/// A completely type-erased index.
pub struct Index {
  lookup: HashTable<(HashValueType, HashSet<EntityId>)>,
}

impl Index {
  pub fn new() -> Self {
    Self {
      lookup: HashTable::new(),
    }
  }

  pub fn insert_with_hash(&mut self, hash: HashValueType, set: HashSet<EntityId>) -> OccupiedEntry<'_, (HashValueType, HashSet<EntityId>)> {
    // > `hasher` is called if entries need to be moved or copied to a new table.
    // > This must return the same hash value that each entry was inserted with.
    let hasher = |(stored_hash, _stored_set): &_| *stored_hash as u64;
    self.lookup.insert_unique(hash as u64, (hash, set), hasher)
  }

  /// The caller is responsible for ensuring that the key has the right type for this index.
  pub fn insert<T: Hash>(&mut self, key: T, set: HashSet<EntityId>) -> OccupiedEntry<'_, (HashValueType, HashSet<EntityId>)> {
    let hash = one_shot_128(&key);
    self.insert_with_hash(hash, set)
  }

  pub fn get_with_hash(&self, hash: HashValueType) -> Option<&HashSet<EntityId>> {
    // Equality is determined by comparing the full 128-bit hashes.
    self.lookup.find(hash as u64, |(stored_hash, _)| *stored_hash == hash).map(|(_, set)| set)
  }

  pub fn get_with_hash_mut(&mut self, hash: HashValueType) -> Option<&mut HashSet<EntityId>> {
    self.lookup.find_mut(hash as u64, |(stored_hash, _)| *stored_hash == hash).map(|(_, set)| set)
  }

  /// The caller is responsible for ensuring that the key has the right type for this index.
  pub fn get<T: Hash>(&self, key: &T) -> Option<&HashSet<EntityId>> {
    let hash = one_shot_128(&key);
    self.get_with_hash(hash)
  }

  /// The caller is responsible for ensuring that the key has the right type for this index.
  pub fn get_mut<T: Hash>(&mut self, key: &T) -> Option<&mut HashSet<EntityId>> {
    let hash = one_shot_128(&key);
    self.get_with_hash_mut(hash)
  }
}
