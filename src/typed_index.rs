/*!

A type-specific struct `Index<T>` with a typed API.

A trait `TypeErasedIndex` defining a type-erased API implemented on all such `Index<T>`.

**Limitations of the type-erased API:**

- Can only "see" values after they have been serialized to some known type (e.g. `String`)
- Cannot create new entries (sets of `PersonId`s), because the `T` value is stored along with the set. As a consequence:
  - Can only insert into existing sets (for values that already have a set)
  - `get_with_hash_mut` is fallible (although we may choose to make the typed version fallible as well)


We might be able to overcome these limitations via serde: the type-erased API only interacts with
serialized values, and internally the value is deserialized. This was the original vision, I think.

*/

use std::any::Any;
use std::collections::HashSet;
use std::hash::Hash;
use hashbrown::{HashTable};
use hashbrown::hash_table::OccupiedEntry;
use crate::hash128::one_shot_128;
use crate::EntityId;

type PersonId = EntityId;

pub type BxIndex = Box<dyn TypeErasedIndex>;
type HashValueType = u128;

/// The typed index.
#[derive(Default)]
pub struct Index<T: Hash + Eq + Clone + Any> {
  // We store a copy of the value here so that we can iterate over it in the typed API, and so that the type-erased
  // API can access some serialization of it.
  lookup: HashTable<(T, HashSet<PersonId>)>,
}

/// Contains the typed API
impl<T: Hash + Eq + Clone + Any> Index<T> {
  pub fn new() -> Self {
    Self {
      lookup: HashTable::default(),
    }
  }

  /// Inserts an entity into the set associated with `key`, creating a new set if one does not yet exist. Returns a
  /// `bool` according to whether the `entity_id` already existed in the set.
  pub fn insert_entity(&mut self, key: &T, entity_id: PersonId) -> bool {
    let hash = one_shot_128(&key);

    // > `hasher` is called if entries need to be moved or copied to a new table.
    // > This must return the same hash value that each entry was inserted with.
    let hasher = |(stored_value, _stored_set): &_| one_shot_128(stored_value) as u64;
    // Equality is determined by comparing the full 128-bit hashes. We do not expect any collisions before the heat
    // death of the universe.
    let hash128_equality = |(stored_value, _): &_| one_shot_128(stored_value) == hash;
    self.lookup
        .entry(hash as u64, hash128_equality, hasher)
        .or_insert_with(|| (key.clone(), HashSet::new()))
        .get_mut()
        .1
        .insert(entity_id)
  }

  /// Inserting a new _value_ requires the value itself.
  pub fn insert_value(&mut self, key: T, set: HashSet<PersonId>) -> OccupiedEntry<'_, (T, HashSet<PersonId>)> {
    let hash = one_shot_128(&key);
    // > `hasher` is called if entries need to be moved or copied to a new table.
    // > This must return the same hash value that each entry was inserted with.
    let hasher = |(stored_value, _stored_set): &_| one_shot_128(stored_value) as u64;
    self.lookup.insert_unique(hash as u64, (key, set), hasher)
  }

  /// Gets an immutable reference to the set associated with the `key` if it exists.
  pub fn get(&self, key: &T) -> Option<&HashSet<PersonId>> {
    let hash = one_shot_128(&key);
    self.get_with_hash(hash)
  }

  /// Gets a mutable reference to the set associated with the `key` if it exists.
  pub fn get_mut(&mut self, key: &T) -> Option<&mut HashSet<PersonId>> {
    let hash = one_shot_128(&key);
    self.get_with_hash_mut(hash)
  }

  pub fn has_key(&self, key: &T) -> bool {
    let hash = one_shot_128(&key);
    self.get_with_hash(hash).is_some()
  }
}


// This trait Encapsulates the type-erased API.
pub trait TypeErasedIndex {
  /// Inserting a new entity only requires the hash but requires the set associated with the hash to already exist.
  ///
  /// If the set corresponding to the hash exists, inserts the `entity_id` into the associated set, returning a `bool`
  /// according to whether the `entity_id` was already in the set.
  /// If the set does not exist, returns `Err(())`
  fn insert_entity_with_hash(&mut self, hash: HashValueType, entity_id: PersonId) -> Result<bool, ()>;

  /// Fetching a set only requires the hash.
  fn get_with_hash(&self, hash: HashValueType) -> Option<&HashSet<PersonId>>;

  /// Fetching a set only requires the hash.
  fn get_with_hash_mut(&mut self, hash: HashValueType) -> Option<&mut HashSet<PersonId>>;

  /// Does the index contain the given hash?
  fn has_hash(&self, hash: HashValueType) -> bool;
}


impl<T: Hash + Eq + Clone + Any> TypeErasedIndex for Index<T> {
  /// Inserting a new entity only requires the hash but requires the set associated with the hash to already exist.
  ///
  /// If the set corresponding to the hash exists, inserts the `entity_id` into the associated set, returning a `bool`
  /// according to whether the `entity_id` was already in the set.
  /// If the set does not exist, returns `Err(())`
  fn insert_entity_with_hash(&mut self, hash: HashValueType, entity_id: PersonId) -> Result<bool, ()> {
    // Equality is determined by comparing the full 128-bit hashes. We do not expect any collisions before the heat
    // death of the universe.
    let hash128_equality = |(stored_value, _): &_| one_shot_128(stored_value) == hash;

    let entities = self.lookup.find_mut(hash as u64, hash128_equality).map(|(_, set)| set).ok_or(())?;
    Ok(entities.insert(entity_id))
  }

  /// Fetching a set only requires the hash.
  fn get_with_hash(&self, hash: HashValueType) -> Option<&HashSet<PersonId>> {
    // Equality is determined by comparing the full 128-bit hashes. We do not expect any collisions before the heat
    // death of the universe.
    let hash128_equality = |(stored_value, _): &_| one_shot_128(stored_value) == hash;
    self.lookup.find(hash as u64, hash128_equality).map(|(_, set)| set)
  }

  /// Fetching a set only requires the hash.
  fn get_with_hash_mut(&mut self, hash: HashValueType) -> Option<&mut HashSet<PersonId>> {
    // Equality is determined by comparing the full 128-bit hashes. We do not expect any collisions before the heat
    // death of the universe.
    let hash128_equality = |(stored_value, _): &_| one_shot_128(stored_value) == hash;
    self.lookup.find_mut(hash as u64, hash128_equality).map(|(_, set)| set)
  }

  fn has_hash(&self, hash: HashValueType) -> bool {
    self.get_with_hash(hash).is_some()
  }
}
