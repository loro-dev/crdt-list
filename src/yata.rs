//! This implement the same algorithm used in the Yjs
//!
//!
//!

use crate::crdt::{ListCrdt, OpSet};

pub trait Yata: ListCrdt {
    fn left_origin(op: &Self::OpUnit) -> Self::OpId;
    fn right_origin(op: &Self::OpUnit) -> Self::OpId;
    fn insert_after(anchor: &mut Self::Cursor<'_>, op: Self::OpUnit);
}

pub fn integrate<T: Yata>(container: &mut T::Container, to_insert: T::OpUnit) {
    let this_left_origin = T::left_origin(&to_insert);
    let this_right_origin = T::right_origin(&to_insert);
    let mut is_first = true;
    let mut cursor = None;
    let mut visited = T::Set::default();
    let mut conflicting_set = T::Set::default();
    for other in T::iter(container, this_left_origin, this_right_origin) {
        if is_first && T::contains(&other, this_left_origin) {
            // skip left origin
            cursor = Some(other);
            is_first = false;
            continue;
        }

        is_first = false;
        if T::contains(&other, this_left_origin) {
            continue;
        }

        if T::contains(&other, this_right_origin) {
            break;
        }

        visited.insert(&other);
        conflicting_set.insert(&other);
        let other_left_origin = T::left_origin(&other);
        if other_left_origin == this_left_origin {
            let other_right_origin = T::right_origin(&other);
            match T::cmp_id(&to_insert, &other) {
                std::cmp::Ordering::Less => {
                    if other_right_origin == this_right_origin {
                        break;
                    }
                }
                std::cmp::Ordering::Greater => {
                    cursor = Some(other);
                    conflicting_set.clear();
                }
                std::cmp::Ordering::Equal => unreachable!(),
            }
        } else if visited.contain(other_left_origin) {
            if !conflicting_set.contain(other_left_origin) {
                cursor = Some(other);
                conflicting_set.clear();
            }
        } else {
            break;
        }
    }

    if let Some(mut cursor) = cursor {
        T::insert_after(&mut cursor, to_insert);
        return;
    }

    drop(cursor);
    T::insert_at(container, to_insert, 0);
}
