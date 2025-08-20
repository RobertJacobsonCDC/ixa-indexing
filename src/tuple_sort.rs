/*!

Type-level sorting of the types in a tuple to produce a new tuple type.

This magic is from https://stackoverflow.com/a/77322510/4492422

*/

use std::marker::PhantomData;

const fn t_greater_than_u<T: 'static, U: 'static>() -> bool {
  let t = unsafe {std::mem::transmute::<_, u128>(std::any::TypeId::of::<T>()) };
  let u = unsafe {std::mem::transmute::<_, u128>(std::any::TypeId::of::<U>()) };
  t > u
}

/// Defines an order on types.
trait TypeComparaison<T> {
  const GREATER: bool;
}

impl<T: 'static, U: 'static> TypeComparaison<U> for T {
  const GREATER: bool = { t_greater_than_u::<T, U>() };
}

// The rest implements bubble sort on the type level.

pub(crate)  trait List {}
pub(crate)  struct Nil;
pub(crate)  struct Cons<Head, Tail: List>(PhantomData<(Head, Tail)>);
impl List for Nil {}
impl<H, T: List> List for Cons<H, T> {}


trait CompareTypeList<L: List> {
  const GREATER: bool;
}
impl<T> CompareTypeList<Nil> for T {
  const GREATER: bool = false;
}
impl<T: 'static, Head: 'static, L: List> CompareTypeList<Cons<Head, L>> for T {
  const GREATER: bool = <T as TypeComparaison<Head>>::GREATER;
}

pub(crate) trait Insert<T, const TYPE_GT: bool> { type Output: List; }
impl<T> Insert<T, true> for Nil { type Output = Cons<T, Nil>; }
impl<T> Insert<T, false> for Nil { type Output = Cons<T, Nil>; }
impl<T, Head, L: List> Insert<T, false> for Cons<Head, L> {
  type Output = Cons<T, Cons<Head, L>>;
}

// T is the type we want to insert in the list. We implement insert for T on Cons<Head, L>, where T > Head.
// So, we need to create a list where Head is first element, and recursively insert T in the tail of the list.
// for this, T must be comparable with the list L, and L must be insertable with T.
impl<T: CompareTypeList<L>, Head, L: List + Insert<T, {<T as CompareTypeList<L>>::GREATER}>> Insert<T, true> for Cons<Head, L> {
  type Output = Cons<Head, <L as Insert<T, {<T as CompareTypeList<L>>::GREATER}>>::Output>;
}


pub(crate) trait SortList { type SortedList: List; }
// how easy it is to sort an empty list !
impl SortList for Nil { type SortedList = Nil; }
impl<Head, L: List + SortList> SortList for Cons<Head, L>
where
    Head: CompareTypeList<<L as SortList>::SortedList>,
    <L as SortList>::SortedList: Insert<Head, {<Head as CompareTypeList<<L as SortList>::SortedList>>::GREATER}>
{
  type SortedList = <<L as SortList>::SortedList as Insert<Head, {<Head as CompareTypeList<<L as SortList>::SortedList>>::GREATER}>>::Output;
}

pub(crate) trait TupleToList {
  type List;
}
pub(crate) trait ListToTuple {
  type Tuple;
}

impl<A> TupleToList for (A, ) {
  type List = Cons<A, Nil>;
}
impl<A> ListToTuple for Cons<A, Nil> {
  type Tuple = (A, );
}

macro_rules! impl_tuple_list_conversions {
    // Base case: implement for a specific length
    ($len:tt => ( $($T:ident),+ ) => $List:ty ) => {
        impl<$($T),+> TupleToList for ( $($T),+ ) {
            type List = $List;
        }

        impl<$($T),+> ListToTuple for $List {
            type Tuple = ( $($T),+ );
        }
    };

    // Repeated cases
    () => {
        impl_tuple_list_conversions!(2 => (A, B) => Cons<A, Cons<B, Nil>>);
        impl_tuple_list_conversions!(3 => (A, B, C) => Cons<A, Cons<B, Cons<C, Nil>>>);
        impl_tuple_list_conversions!(4 => (A, B, C, D) => Cons<A, Cons<B, Cons<C, Cons<D, Nil>>>>);
        // Add more as needed...
    };
}

impl_tuple_list_conversions!();


pub(crate) trait TupleSortTrait {
  type Sorted;
}

impl<T> TupleSortTrait for T
where T: TupleToList,
      <T as TupleToList>::List: SortList,
      <<T as TupleToList>::List as SortList>::SortedList: ListToTuple,
{
  type Sorted = <<<T as TupleToList>::List as SortList>::SortedList as ListToTuple>::Tuple;
}

pub(crate) type TupleSort<T> = <T as TupleSortTrait>::Sorted;


pub(crate) trait Permute<T> where T: TupleSortTrait {
  fn sorted(self) -> T::Sorted;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_sort_tuple() {
    let v = vec![
      <(u8, String, bool) as Default>::default(),
      <TupleSort<(String, bool, u8)> as Default>::default(),
      <TupleSort<(bool, String, u8)> as Default>::default(),
    ];
    println!("Our vector:\n[");
    for v in v {
      println!("\t{:?},", v);
    }
    println!("]");
  }
}
