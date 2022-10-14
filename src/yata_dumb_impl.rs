use std::collections::HashSet;

use crate::{
    crdt::ListCrdt,
    dumb_common::{Container, Cursor, Iter, Op, OpId, OpSetImpl},
    test::TestFramework,
    yata,
};

impl YataImpl {
    fn container_contains(
        container: &<Self as ListCrdt>::Container,
        op_id: Option<<Self as ListCrdt>::OpId>,
    ) -> bool {
        if op_id.is_none() {
            return true;
        }
        let op_id = op_id.unwrap();
        container.content.iter().any(|x| x.id == op_id)

        // if container.version_vector.len() <= op_id.client_id {
        //     return false;
        // }

        // container.version_vector[op_id.client_id] > op_id.clock
    }
}

pub struct YataImpl;
impl ListCrdt for YataImpl {
    type OpUnit = Op;

    type OpId = OpId;

    type Container = Container;

    type Cursor<'a> = Cursor<'a>;

    type Set = OpSetImpl;

    type Iterator<'a> = Iter<'a>;

    fn iter(
        container: &mut Self::Container,
        from: Option<Self::OpId>,
        to: Option<Self::OpId>,
    ) -> Self::Iterator<'_> {
        Iter {
            arr: &mut container.content,
            index: 0,
            start: from,
            end: to,
            done: false,
            started: false,
            exclude_end: true,
        }
    }

    fn insert_at(container: &mut Self::Container, op: Self::OpUnit, pos: usize) {
        container.content.insert(pos, op);
    }

    fn id(op: &Self::OpUnit) -> Self::OpId {
        op.id
    }

    fn cmp_id(op_a: &Self::OpUnit, op_b: &Self::OpUnit) -> std::cmp::Ordering {
        op_a.id
            .client_id
            .cmp(&op_b.id.client_id)
            .then(op_a.id.clock.cmp(&op_b.id.clock))
    }

    fn contains(op: &Self::OpUnit, id: Self::OpId) -> bool {
        op.id == id
    }

    fn integrate(container: &mut Self::Container, op: Self::OpUnit) {
        let id = Self::id(&op);
        for _ in container.version_vector.len()..id.client_id + 1 {
            container.version_vector.push(0);
        }
        assert!(container.version_vector[id.client_id] == id.clock);
        unsafe { yata::integrate::<YataImpl>(container, op) };

        container.version_vector[id.client_id] = id.clock + 1;
    }

    fn can_integrate(container: &Self::Container, op: &Self::OpUnit) -> bool {
        Self::container_contains(container, op.left)
            && Self::container_contains(container, op.right)
            && (op.id.clock == 0
                || Self::container_contains(
                    container,
                    Some(OpId {
                        client_id: op.id.client_id,
                        clock: op.id.clock - 1,
                    }),
                ))
    }

    fn len(container: &Self::Container) -> usize {
        container.content.len()
    }
}

impl yata::Yata for YataImpl {
    fn left_origin(op: &Self::OpUnit) -> Option<Self::OpId> {
        op.left
    }

    fn right_origin(op: &Self::OpUnit) -> Option<Self::OpId> {
        op.right
    }

    fn insert_after(_: &mut Self::Container, anchor: Self::Cursor<'_>, op: Self::OpUnit) {
        if anchor.pos + 1 >= anchor.arr.len() {
            anchor.arr.push(op);
        } else {
            anchor.arr.insert(anchor.pos + 1, op);
        }
    }

    fn insert_after_id(container: &mut Self::Container, id: Option<Self::OpId>, op: Self::OpUnit) {
        if let Some(id) = id {
            let pos = container.content.iter().position(|x| x.id == id).unwrap();
            container.content.insert(pos + 1, op);
        } else {
            Self::insert_at(container, op, 0)
        }
    }
}

impl TestFramework for YataImpl {
    fn is_content_eq(a: &Self::Container, b: &Self::Container) -> bool {
        match a.content.eq(&b.content) {
            true => true,
            false => {
                dbg!(&a.content);
                dbg!(&b.content);
                false
            }
        }
    }

    fn new_container(id: usize) -> Self::Container {
        Container {
            id,
            version_vector: vec![0; 10],
            ..Default::default()
        }
    }

    fn new_op(
        _rng: &mut impl rand::Rng,
        container: &mut Self::Container,
        pos: usize,
    ) -> Self::OpUnit {
        let insert_pos = pos % (container.content.len() + 1);
        let (left, right) = if container.content.is_empty() {
            (None, None)
        } else if insert_pos == 0 {
            (None, Some(container.content[0].id))
        } else if insert_pos == container.content.len() {
            (Some(container.content[insert_pos - 1].id), None)
        } else {
            (
                Some(container.content[insert_pos - 1].id),
                Some(container.content[insert_pos].id),
            )
        };

        let ans = Op {
            id: OpId {
                client_id: container.id,
                clock: container.max_clock,
            },
            left,
            right,
            deleted: false,
        };

        container.max_clock += 1;
        ans
    }

    type DeleteOp = HashSet<Self::OpId>;

    fn new_del_op(container: &Self::Container, pos: usize, len: usize) -> Self::DeleteOp {
        let mut deleted = HashSet::new();
        for op in container.content.iter_real().skip(pos).take(len) {
            deleted.insert(op.id);
        }

        deleted
    }

    fn integrate_delete_op(container: &mut Self::Container, delete_set: Self::DeleteOp) {
        for op in container.content.iter_real_mut() {
            if delete_set.contains(&op.id) {
                op.deleted = true;
            }
        }
    }
}

#[cfg(test)]
mod yata_impl_test {
    use super::*;

    #[test]
    fn run() {
        for seed in 0..100 {
            crate::test::test::<YataImpl>(seed, 2, 1000);
        }
    }

    #[test]
    fn run_3() {
        for seed in 0..100 {
            crate::test::test::<YataImpl>(seed, 3, 1000);
        }
    }

    #[test]
    fn run_10() {
        for seed in 0..100 {
            crate::test::test::<YataImpl>(seed, 10, 1000);
        }
    }

    use ctor::ctor;
    #[ctor]
    fn init_color_backtrace() {
        color_backtrace::install();
    }
}
