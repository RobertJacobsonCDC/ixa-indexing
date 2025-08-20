/*!

For multi-indexes, we create a derived property with values of a tuple type. The
tuple types are always in sorted order. We achieve this by using a proc macro. If we
want to preserve the original order, we have to convert between the sorted tuple type
and the original tuple type. The proc macro will generate the implementation of the
conversion functions.

We need to sort by the property names instead of the property value types. (See notes on
`SortableTuple`.) We can achieve this by implementing `SortByTag<Tag>` for the value tuple
type (with a proc macro).

There are other ways to implement multi-indexes. For example, we could generate a struct
with fields corresponding to the property names and values corresponding to the property
values. There is type-level magic that allows us to convert between types with the same
fields but different names. But this feels more complicated than the proc macro approach.

*/

trait Property {
  type Value;
}

/**
If we are ok with having the canonical sort order of properties be the order of their corresponding types,
we can implement `SortableTuple` for the value tuple type.

EDIT: Actually, this doesn't work, because the value tuple type might be `(u8, u8, u8)`, which tells you
nothing about the order of the properties.
*/
trait SortableTuple {
  type Sorted;

  fn to_sorted_tuple(self) -> Self::Sorted;
  fn from_sorted_tuple(sorted: Self::Sorted) -> Self;
}

/**
If we want to sort by the property names instead of the property value types, we instead implement
`SortByTag` for the value tuple type. Note that we don't need to implement reordering methods for
the tag tuple, because `Tag` is the unordered value, and `SortByTag<Tag>::SortedTag` is the ordered
value.
*/
trait SortByTag<Tag> {
  type SortedTag;
  type ReorderedValue;

  fn reorder_by_tag(self) -> Self::ReorderedValue;
  /// The inverse of `reorder_by_tag`. Note that this is an associated function, not a method.
  fn unreorder_by_tag(sorted: Self::ReorderedValue) -> Self;
}



#[cfg(test)]
mod tests {
  use super::*;
  use ixa_derive::sorted_tuple_impl;

  sorted_tuple_impl!(usize, f64, &'static str);

  #[test]
  fn test_sorted_tuple() {
    let a = (1, 2.0, "a");
    let sorted = a.to_sorted_tuple();
    let unsorted: (usize, f64, &'static str) = SortableTuple::from_sorted_tuple(sorted);

    let expected = ("a", 2.0, 1);

    assert_eq!(a, unsorted);
    assert_eq!(sorted, expected);

    println!("original: {:?}, sorted: {:?}", a, sorted);
  }


  struct TagA;
  struct TagB;
  struct TagC;

  use ixa_derive::sorted_tag_value_impl;

  // The macro is written generically in case we have other use cases. For multi-indexes,
  // we would generate the following:
  // ```rust,ignore
  //   sorted_tag_value_impl!(
  //     tag_tuple = (TagC, TagA, TagB),
  //     value_tuple = (TagC::Value, TagA::Value, TagB::Value)
  //   );
  // ```
  sorted_tag_value_impl!(
    tag_tuple = (TagC, TagA, TagB),
    value_tuple = (u8, &'static str, f64)
  );
  /*
  // The macro above generates the following:
  impl SortByTag<(TagC, TagA, TagB)> for (u8, &'static str, f64) {
    type SortedTag = (TagA, TagB, TagC);
    type ReorderedValue = (&'static str, f64, u8);
    fn reorder_by_tag(self) -> Self::ReorderedValue {
      let (t0, t1, t2) = self;
      (t1, t2, t0)
    }
    fn unreorder_by_tag(sorted: Self::ReorderedValue) -> Self {
      let (s0, s1, s2) = sorted;
      (s2, s0, s1)
    }
  }
  */

  #[test]
  fn test_sort_by_tag() {
    let values = (123u8, "hi", 3.14);
    let sorted = values.reorder_by_tag();
    // You would need a type annotation if the types were ambiguous.
    // let sorted = <_ as SortByTag<(TagC, TagA, TagB)>>::reorder_by_tag(values);
    let expected_sorted = ("hi", 3.14, 123);
    let unsorted = <(u8, &'static str, f64)>::unreorder_by_tag(expected_sorted);
    // You would need the full type annotation if the types were ambiguous.
    // let unsorted = <(u8, &'static str, f64) as SortByTag<(TagC, TagA, TagB)>>::unreorder_by_tag(expected_sorted);

    println!("expected: {:?}\nsorted:   {:?}\nunsorted: {:?}", expected_sorted, sorted, unsorted);

    assert_eq!(sorted, expected_sorted);
    assert_eq!(unsorted, values);
  }


}
