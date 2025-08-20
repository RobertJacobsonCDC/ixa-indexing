/*!

Requirements:

- An index for a property of arbitrary static type.
- Index is fetchable via type id (or name).
- The index maps values of the type to a set of people.
- Multi-index is independent of sort order.

Nice to have:
- Can iterate over key-value pairs. This requires a way to store the key in a type-erased way.


*/

// #![feature(generic_const_exprs)]
// #![feature(const_type_id)]
#![allow(dead_code)]
use std::any::TypeId;
use std::collections::HashMap;
use crate::typed_index::BxIndex;

mod type_erased_index;
mod hash128;
mod typed_index;
mod multi_index;
// mod tuple_sort;

pub type EntityId = u64;

struct PropertyManager {
  /// Resolves property names to type ids.
  property_names: HashMap<&'static str, TypeId>,
  indexes       : HashMap<TypeId, BxIndex>,

}
