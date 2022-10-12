use crate::crdt::{GetOp, ListCrdt, OpSet};

pub trait Woot: ListCrdt {
    fn left(op: &Self::OpUnit) -> Option<Self::OpId>;
    fn right(op: &Self::OpUnit) -> Option<Self::OpId>;
    fn get_pos_of(container: &Self::Container, op_id: Self::OpId) -> usize;
}

pub fn integrate<T: Woot>(
    container: &mut T::Container,
    to_insert: T::OpUnit,
    left: Option<T::OpId>,
    right: Option<T::OpId>,
) {
    let mut set = T::Set::default();
    let mut empty_between_left_and_right = true;
    for ref op in T::iter(container, left, right) {
        let op = &op.get_op();
        if (left.is_some() && T::contains(op, left.unwrap()))
            || (right.is_some() && T::contains(op, right.unwrap()))
        {
            continue;
        }

        empty_between_left_and_right = false;
        set.insert(op);
    }

    if empty_between_left_and_right {
        match right {
            Some(right) => T::insert_at(container, to_insert, T::get_pos_of(container, right)),
            None => T::insert_at(container, to_insert, T::len(container)),
        }
        return;
    }

    let mut prev = left;
    let mut next = right;
    for ref iter_op in T::iter(container, left, right).filter(|op| {
        let op = &op.get_op();
        let left = T::left(op);
        let right = T::right(op);
        (left.is_none() || !set.contain(left.unwrap()))
            && (right.is_none() || !set.contain(right.unwrap()))
    }) {
        let iter_op = &iter_op.get_op();
        if Some(T::id(iter_op)) == left || Some(T::id(iter_op)) == right {
            // left cannot be next, and right cannot be prev
            continue;
        }

        if T::cmp_id(iter_op, &to_insert).is_lt() {
            prev = Some(T::id(iter_op));
        } else {
            next = Some(T::id(iter_op));
            break;
        }
    }

    integrate::<T>(container, to_insert, prev, next);
}
