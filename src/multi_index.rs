

trait SortedTuple {
  type Sorted;

  fn to_sorted_tuple(self) -> Self::Sorted;
  fn from_sorted_tuple(sorted: Self::Sorted) -> Self;
}


macro_rules! define_multi_index {
  ( $( $name:ident ),* ) => (

    define_derived_property!( TupleSort<($( $name ),*)>, [$( $name ),*], [], |$( $name ),*)| (
      $( $name ),* )
  )
}
