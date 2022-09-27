use std::{cmp::Ordering, fmt::Debug};

pub trait OpSet<Op, OpId>: Default {
    fn insert(&mut self, value: &Op);
    fn contain(&self, id: OpId) -> bool;
}

pub trait Crdt {
    type OpUnit: Clone + Debug;
    type OpId: Eq + Copy + Debug;
    type Container: Debug;
    type Set: OpSet<Self::OpUnit, Self::OpId>;
    type Iterator<'a>: Iterator<Item = (usize, &'a Self::OpUnit)>
    where
        <Self as Crdt>::OpUnit: 'a;

    /// inclusive end
    fn iter(container: &Self::Container, from: Self::OpId, to: Self::OpId) -> Self::Iterator<'_>;
    fn insert_at(container: &mut Self::Container, op: Self::OpUnit, pos: usize);
    fn id(op: &Self::OpUnit) -> Self::OpId;
    fn cmp(op_a: &Self::OpUnit, op_b: &Self::OpUnit) -> Ordering;
    fn contains(op: &Self::OpUnit, id: Self::OpId) -> bool;
    fn integrate(container: &mut Self::Container, op: Self::OpUnit);
    fn can_integrate(container: &Self::Container, op: &Self::OpUnit) -> bool;
}
