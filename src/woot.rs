use crate::crdt::{ListCrdt, OpSet};

pub trait Woot: ListCrdt {
    fn left(op: &Self::OpUnit) -> Self::OpId;
    fn right(op: &Self::OpUnit) -> Self::OpId;
    fn get_pos_of(container: &Self::Container, op_id: Self::OpId) -> usize;
}

pub fn integrate<T: Woot>(
    container: &mut T::Container,
    to_insert: T::OpUnit,
    left: T::OpId,
    right: T::OpId,
) {
    let mut set = T::Set::default();
    let mut empty_between_left_and_right = true;
    for (_, op) in T::iter(container, left, right) {
        if T::contains(op, left) || T::contains(op, right) {
            continue;
        }

        empty_between_left_and_right = false;
        set.insert(op);
    }

    if empty_between_left_and_right {
        T::insert_at(container, to_insert, T::get_pos_of(container, right));
        return;
    }

    let mut prev = left;
    let mut next = right;
    for (_, iter_op) in T::iter(container, left, right)
        .filter(|(_, op)| !set.contain(T::left(op)) && !set.contain(T::right(op)))
    {
        if T::id(iter_op) == left || T::id(iter_op) == right {
            // left cannot be next, and right cannot be prev
            continue;
        }

        if T::cmp(iter_op, &to_insert).is_lt() {
            prev = T::id(iter_op);
        } else {
            next = T::id(iter_op);
            break;
        }
    }

    integrate::<T>(container, to_insert, prev, next);
}
