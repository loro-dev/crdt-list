//! This implement the same algorithm used in the Yjs
//!
//!
//!

use std::ptr::NonNull;

use crate::crdt::{ListCrdt, OpSet};

pub trait Yata: ListCrdt {
    fn left_origin(op: &Self::OpUnit) -> Option<Self::OpId>;
    fn right_origin(op: &Self::OpUnit) -> Option<Self::OpId>;
    fn insert_after(container: &mut Self::Container, anchor: Self::Cursor<'_>, op: Self::OpUnit);
}

/// # Safety
///
/// Users should be sure that inside [`Yata::insert_after`] there are not more than one
/// exclusive references to the same element
pub unsafe fn integrate<T: Yata>(container: &mut T::Container, to_insert: T::OpUnit) {
    let this_left_origin = T::left_origin(&to_insert);
    let this_right_origin = T::right_origin(&to_insert);
    let mut cursor = None;
    let mut visited = T::Set::default();
    let mut conflicting_set = T::Set::default();
    let mut container_ptr: NonNull<_> = container.into();
    for other in T::iter(container, this_left_origin, this_right_origin) {
        if this_left_origin.is_some() && T::contains(&other, this_left_origin.unwrap()) {
            // skip left origin
            cursor = Some(other);
            continue;
        }

        if this_right_origin.is_some() && T::contains(&other, this_right_origin.unwrap()) {
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
        } else if other_left_origin.is_some() && visited.contain(other_left_origin.unwrap()) {
            if !conflicting_set.contain(other_left_origin.unwrap()) {
                cursor = Some(other);
                conflicting_set.clear();
            }
        } else {
            break;
        }
    }

    if let Some(cursor) = cursor {
        let container = container_ptr.as_mut();
        T::insert_after(container, cursor, to_insert);
        return;
    }

    drop(cursor);

    if this_left_origin.is_none() {
        T::insert_at(container, to_insert, 0);
    } else {
        panic!("Cannot find the insert position");
    }
}
