//! This implement the same algorithm used in the Yjs
//!
//!
//!

use crate::crdt::{GetOp, ListCrdt, OpSet};

/// For Yata iter should only iterate over the element between `start` and `to`, exclude both `start` and `to`
pub trait Yata: ListCrdt {
    type Context;
    fn left_origin(op: &Self::OpUnit) -> Option<Self::OpId>;
    fn right_origin(op: &Self::OpUnit) -> Option<Self::OpId>;
    /// insert after the anchor
    fn insert_after(anchor: Self::Cursor<'_>, op: Self::OpUnit, context: &mut Self::Context);
    fn insert_after_id(
        container: &mut Self::Container,
        id: Option<Self::OpId>,
        op: Self::OpUnit,
        context: &mut Self::Context,
    );
}

pub fn integrate<T: Yata>(
    container: &mut T::Container,
    to_insert: T::OpUnit,
    ctx: &mut T::Context,
) {
    let this_left_origin = T::left_origin(&to_insert);
    let this_right_origin = T::right_origin(&to_insert);
    let mut cursor = None;
    let mut visited = T::Set::default();
    let mut conflicting_set = T::Set::default();
    for other_cursor in T::iter(container, this_left_origin, this_right_origin) {
        let other = other_cursor.get_op();
        if (this_left_origin.is_some() && T::contains(&other, this_left_origin.unwrap()))
            || (this_right_origin.is_some() && T::contains(&other, this_right_origin.unwrap()))
        {
            unreachable!("For Yata iter should only iterate over the element between `start` and `to`, exclude both `start` and `to`");
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
        T::insert_after(cursor, to_insert, ctx);
        return;
    }

    drop(cursor);
    T::insert_after_id(container, this_left_origin, to_insert, ctx);
}
