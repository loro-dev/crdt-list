//! This implement the same algorithm used in the Yjs
//!
//!
//!

use std::ptr::NonNull;

use crate::crdt::{GetOp, ListCrdt, OpSet};

pub trait Yata: ListCrdt {
    fn left_origin(op: &Self::OpUnit) -> Option<Self::OpId>;
    fn right_origin(op: &Self::OpUnit) -> Option<Self::OpId>;
    /// insert after the anchor
    fn insert_after(container: &mut Self::Container, anchor: Self::Cursor<'_>, op: Self::OpUnit);

    /// insert right after the anchor.
    ///
    /// [Yata::insert_after] and [Yata::insert_immediately_after] are the same if the cursor length is 1.
    ///
    /// When cursor has length greater than 1
    /// - [Yata::insert_immediately_after] should insert at the position of cursor.start + 1
    /// - [Yata::insert_after] should insert at the position of cursor.start + cursor.len
    fn insert_immediately_after(
        container: &mut Self::Container,
        anchor: Self::Cursor<'_>,
        op: Self::OpUnit,
    );
}

/// # Safety
///
/// Users should be sure that inside [`Yata::insert_after`] there are not more than one
/// exclusive references to the same element
pub unsafe fn integrate<T: Yata>(container: &mut T::Container, to_insert: T::OpUnit) {
    let this_left_origin = T::left_origin(&to_insert);
    let this_right_origin = T::right_origin(&to_insert);
    let mut first_cursor = None;
    let mut cursor = None;
    let mut visited = T::Set::default();
    let mut conflicting_set = T::Set::default();
    let mut container_ptr: NonNull<_> = container.into();
    for other_cursor in T::iter(container, this_left_origin, this_right_origin) {
        let other = other_cursor.get_op();
        if this_left_origin.is_some() && T::contains(&other, this_left_origin.unwrap()) {
            // skip left origin
            first_cursor = Some(other_cursor);
            continue;
        }

        if this_right_origin.is_some() && T::contains(&other, this_right_origin.unwrap()) {
            break;
        }

        visited.insert(&other);
        conflicting_set.insert(&other);
        let other_left_origin = T::left_origin(&other);
        if other_left_origin == this_left_origin {
            match T::cmp_id(&to_insert, &other) {
                std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                    let other_right_origin = T::right_origin(&other);
                    if other_right_origin == this_right_origin {
                        break;
                    }
                }
                std::cmp::Ordering::Greater => {
                    cursor = Some(other_cursor);
                    conflicting_set.clear();
                }
            }
        } else if other_left_origin.is_some() && visited.contain(other_left_origin.unwrap()) {
            if !conflicting_set.contain(other_left_origin.unwrap()) {
                cursor = Some(other_cursor);
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
    if let Some(cursor) = first_cursor {
        let container = container_ptr.as_mut();
        T::insert_immediately_after(container, cursor, to_insert);
        return;
    }

    drop(cursor);
    drop(first_cursor);

    if this_left_origin.is_none() {
        T::insert_at(container, to_insert, 0);
    } else {
        panic!("Cannot find the insert position");
    }
}
