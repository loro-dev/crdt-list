use std::{cmp::Ordering, fmt::Debug, ops::Deref};

pub trait OpSet<Op, OpId>: Default {
    fn insert(&mut self, value: &Op);
    fn contain(&self, id: OpId) -> bool;
    fn clear(&mut self);
}

pub trait ListCrdt {
    type OpUnit: Clone + Debug;
    type OpId: Eq + Copy + Debug;
    type Container: Debug;
    type Set: OpSet<Self::OpUnit, Self::OpId>;
    type Cursor<'a>: Deref<Target = Self::OpUnit>;
    type Iterator<'a>: Iterator<Item = Self::Cursor<'a>>
    where
        <Self as ListCrdt>::OpUnit: 'a,
        <Self as ListCrdt>::Cursor<'a>: 'a;

    /// inclusive end
    fn iter(
        container: &mut Self::Container,
        from: Option<Self::OpId>,
        to: Option<Self::OpId>,
    ) -> Self::Iterator<'_>;
    fn insert_at(container: &mut Self::Container, op: Self::OpUnit, pos: usize);
    fn id(op: &Self::OpUnit) -> Self::OpId;
    fn cmp_id(op_a: &Self::OpUnit, op_b: &Self::OpUnit) -> Ordering;
    fn contains(op: &Self::OpUnit, id: Self::OpId) -> bool;
    fn integrate(container: &mut Self::Container, op: Self::OpUnit);
    fn can_integrate(container: &Self::Container, op: &Self::OpUnit) -> bool;
    fn len(container: &Self::Container) -> usize;
}
