use std::collections::HashSet;

use crate::{
    crdt::ListCrdt,
    dumb_common::{Container, Cursor, Iter, Op, OpId, OpSetImpl},
    test::TestFramework,
    woot,
};
use rand::Rng;

impl WootImpl {
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

pub struct WootImpl;
impl ListCrdt for WootImpl {
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
            exclude_end: false,
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
        woot::integrate::<WootImpl>(container, op.clone(), op.left, op.right);

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

impl woot::Woot for WootImpl {
    fn left(op: &Self::OpUnit) -> Option<Self::OpId> {
        op.left
    }

    fn right(op: &Self::OpUnit) -> Option<Self::OpId> {
        op.right
    }

    fn get_pos_of(container: &Container, op_id: Self::OpId) -> usize {
        container
            .content
            .iter()
            .position(|x| x.id == op_id)
            .unwrap()
    }
}

impl TestFramework for WootImpl {
    fn is_content_eq(a: &Self::Container, b: &Self::Container) -> bool {
        a.content.eq(&b.content)
    }

    fn new_container(id: usize) -> Self::Container {
        Container {
            id,
            version_vector: vec![0; 10],
            ..Default::default()
        }
    }

    fn new_op(_rng: &mut impl Rng, container: &mut Self::Container, pos: usize) -> Self::OpUnit {
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

    fn new_del_op(container: &Self::Container, mut pos: usize, mut len: usize) -> Self::DeleteOp {
        let content_len = container.content.real_len();
        let mut deleted = HashSet::new();
        if content_len == 0 {
            return deleted;
        }

        pos %= content_len;
        len = std::cmp::min(len, content_len - pos);
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
mod woot_impl_test {
    use super::*;

    #[test]
    fn run() {
        for i in 0..100 {
            crate::test::test::<WootImpl>(i, 2, 1000);
        }
    }

    #[test]
    fn run3() {
        for seed in 0..100 {
            crate::test::test::<WootImpl>(seed, 3, 1000);
        }
    }

    #[test]
    fn run_n() {
        for n in 2..10 {
            crate::test::test::<WootImpl>(123, n, 10000);
        }
    }

    use ctor::ctor;
    #[ctor]
    fn init_color_backtrace() {
        color_backtrace::install();
    }
}
