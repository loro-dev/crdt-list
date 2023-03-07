use crate::crdt::{GetOp, ListCrdt};

pub trait Rga: ListCrdt {
    type Lamport: Ord;
    type ClientId: Ord;
    fn left(op: &Self::OpUnit) -> Option<Self::OpId>;
    fn client_id(id: Self::OpId) -> Self::ClientId;
    fn lamport(op: &Self::OpUnit) -> Self::Lamport;
    fn len(container: &Self::Container) -> usize;
    fn insert_after(container: &mut Self::Container, left: Option<Self::OpId>, op: Self::OpUnit);
}

pub fn integrate<T: Rga>(container: &mut T::Container, to_insert: T::OpUnit) {
    let origin_left = T::left(&to_insert);
    let client_id = T::client_id(T::id(&to_insert));
    let lamport = T::lamport(&to_insert);
    let cmp = (lamport, client_id);
    let mut left = None;
    for op in T::iter(container, origin_left, None) {
        let op = op.get_op();
        let op_client = T::client_id(T::id(&op));
        if cmp < (T::lamport(&op), op_client) {
            break;
        }
        left = Some(T::id(&op));
    }

    T::insert_after(container, left, to_insert);
}
