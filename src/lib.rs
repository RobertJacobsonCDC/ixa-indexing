/*!

Requirements:

- An index for a property of arbitrary static type.
- Index is fetchable via type id (or name).
- The index maps values of the type to a set of people.
- Multi-index is independent of sort order.

Nice to have:
- Can iterate over key-value pairs. This requires a way to store the key in a type-erased way.


*/

mod type_erased_index;
mod hash128;
mod typed_index;

pub type EntityId = u64;
