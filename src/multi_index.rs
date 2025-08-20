/*!

For multi-indexes, we create a derived trait with values of a tuple type. The tuple types are always in sorted order. We
achieve this by using a proc macro. If we want to preserve the original order, we have to convert between the sorted
tuple type and the original tuple type. The proc macro will generate the implementation of the conversion functions
`SortedTuple::to_sorted_tuple` and `SortedTuple::from_sorted_tuple`.

*/

trait SortableTuple {
  type Sorted;

  fn to_sorted_tuple(self) -> Self::Sorted;
  fn from_sorted_tuple(sorted: Self::Sorted) -> Self;
}



#[cfg(test)]
mod tests {
  use super::*;
  use ixa_derive::sorted_tuple_impl;

  sorted_tuple_impl!(usize, f64, &'static str);

  #[test]
  fn it_works() {
    let a = (1, 2.0, "a");
    let sorted = a.to_sorted_tuple();
    let unsorted: (usize, f64, &'static str) = SortableTuple::from_sorted_tuple(sorted);

    let expected = ("a", 2.0, 1);

    assert_eq!(a, unsorted);
    assert_eq!(sorted, expected);

    println!("original: {:?}, sorted: {:?}", a, sorted);
  }
}
