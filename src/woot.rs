use std::cmp::Ordering;

use crate::crdt::{Crdt, OpSet};

pub trait Woot: Crdt {
    fn left(op: &Self::OpUnit) -> Self::OpId;
    fn right(op: &Self::OpUnit) -> Self::OpId;
    fn get_pos_of(container: &Self::Container, op_id: Self::OpId) -> usize;
}

pub fn integrate<T: Woot>(
    container: &mut T::Container,
    value: T::OpUnit,
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
        T::insert_at(container, value, T::get_pos_of(container, right));
        return;
    }

    let mut prev = left;
    let mut next = right;
    for (_, op) in T::iter(container, left, right)
        .filter(|(_, op)| !set.contain(T::left(op)) && !set.contain(T::right(op)))
    {
        if T::id(op) == prev {
            continue;
        }

        if T::cmp(op, &value).is_lt() {
            prev = T::id(op);
        } else {
            next = T::id(op);
            break;
        }
    }

    if prev == left && next == right {
        dbg!(container);
        panic!("Something is off");
    }

    integrate::<T>(container, value, prev, next);
}
