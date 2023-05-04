//! This mod impl The Art of the Fugue: Minimizing Interleaving in Collaborative Text Editing
//!

use std::cmp::Ordering;

use crate::crdt::{GetOp, ListCrdt, OpSet};

/// For Fugue, iter should only iterate over the element between `start` and `to`, exclude both `start` and `to`
pub trait Fugue: ListCrdt {
    type Context;
    fn left_origin(op: &Self::OpUnit) -> Option<Self::OpId>;
    fn left_origin_of_id(container: &Self::Container, op_id: &Self::OpId) -> Option<Self::OpId>;
    fn right_origin(op: &Self::OpUnit) -> Option<Self::OpId>;
    /// insert after the anchor
    fn insert_after(anchor: Self::Cursor<'_>, op: Self::OpUnit, context: &mut Self::Context);
    fn insert_after_id(
        container: &mut Self::Container,
        id: Option<Self::OpId>,
        op: Self::OpUnit,
        context: &mut Self::Context,
    );
    fn cmp_pos(
        container: &Self::Container,
        op_a: Option<Self::OpId>,
        op_b: Option<Self::OpId>,
    ) -> Ordering;
}

pub fn integrate<T: Fugue>(
    container: &mut T::Container,
    to_insert: T::OpUnit,
    ctx: &mut T::Context,
) {
    let this_left_origin = T::left_origin(&to_insert);
    let this_right_origin = T::right_origin(&to_insert);
    let this_right_parent = this_right_origin.and_then(|x| {
        if T::left_origin_of_id(container, &x) == this_left_origin {
            Some(x)
        } else {
            None
        }
    });
    let mut cursor = None;
    let mut visited = T::Set::default();
    let mut scanning = false;

    for other_cursor in T::iter(
        unsafe { std::mem::transmute(&mut *container) },
        this_left_origin,
        this_right_origin,
    ) {
        let other = other_cursor.get_op();
        if (this_left_origin.is_some() && T::contains(&other, this_left_origin.unwrap()))
            || (this_right_origin.is_some() && T::contains(&other, this_right_origin.unwrap()))
        {
            unreachable!("For Fugue iter should only iterate over the element between `start` and `to`, exclude both `start` and `to`");
        }

        let o_left_origin = T::left_origin(&other);

        // o.leftOrigin < elt.leftOrigin (< compares the position)
        if o_left_origin.map(|x| !visited.contain(x)).unwrap_or(true)
            && o_left_origin != this_left_origin
        {
            break;
        }

        visited.insert(&other);
        if o_left_origin == this_left_origin {
            let o_right_parent = T::right_origin(&other).and_then(|x| {
                if T::left_origin_of_id(container, &x) == this_left_origin {
                    Some(x)
                } else {
                    None
                }
            });

            match T::cmp_pos(container, o_right_parent, this_right_parent) {
                Ordering::Less => {
                    scanning = true;
                }
                Ordering::Equal if T::cmp_id(&other, &to_insert) == Ordering::Greater => {
                    break;
                }
                _ => {
                    scanning = false;
                }
            }
        }

        if !scanning {
            cursor = Some(other_cursor);
        }
    }

    if let Some(cursor) = cursor {
        T::insert_after(cursor, to_insert, ctx);
        return;
    }

    drop(cursor);
    T::insert_after_id(container, this_left_origin, to_insert, ctx);
}
