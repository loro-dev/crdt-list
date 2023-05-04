use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

pub use crate::dumb_common::{Container, Cursor, Iter, Op, OpId, OpSetImpl};
use crate::{crdt::ListCrdt, rga, test::TestFramework};
use rand::Rng;

pub struct RgaImpl;
impl RgaImpl {
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

#[derive(Debug)]
pub struct RgaContainer {
    container: Container,
    next_lamport: u32,
}

impl Deref for RgaContainer {
    type Target = Container;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl DerefMut for RgaContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.container
    }
}

impl ListCrdt for RgaImpl {
    type OpUnit = Op;

    type OpId = OpId;

    type Container = RgaContainer;

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
}

impl rga::Rga for RgaImpl {
    fn len(container: &Self::Container) -> usize {
        container.content.len()
    }

    fn left(op: &Self::OpUnit) -> Option<Self::OpId> {
        op.left
    }

    type Lamport = u32;

    fn lamport(op: &Self::OpUnit) -> Self::Lamport {
        op.lamport
    }

    fn insert_after(container: &mut Self::Container, left: Option<Self::OpId>, op: Self::OpUnit) {
        match left {
            Some(left) => {
                let pos = container.content.iter().position(|x| x.id == left).unwrap();
                container.content.insert(pos + 1, op);
            }
            None => {
                container.content.insert(0, op);
            }
        }
    }

    type ClientId = usize;

    fn client_id(id: Self::OpId) -> Self::ClientId {
        id.client_id
    }
}

impl TestFramework for RgaImpl {
    fn is_content_eq(a: &Self::Container, b: &Self::Container) -> bool {
        a.content.eq(&b.content)
    }

    fn new_container(id: usize) -> Self::Container {
        RgaContainer {
            container: Container {
                id,
                version_vector: vec![0; 10],
                ..Default::default()
            },
            next_lamport: 0,
        }
    }

    fn new_op(container: &mut Self::Container, pos: usize) -> Self::OpUnit {
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
            lamport: container.next_lamport,
        };

        container.max_clock += 1;
        container.next_lamport += 1;
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

    fn integrate(container: &mut Self::Container, op: Self::OpUnit) {
        container.next_lamport = std::cmp::max(container.next_lamport, op.lamport + 1);
        let id = Self::id(&op);
        for _ in container.version_vector.len()..id.client_id + 1 {
            container.version_vector.push(0);
        }
        assert!(container.version_vector[id.client_id] == id.clock);
        rga::integrate::<RgaImpl>(container, op);

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
}

#[cfg(test)]
mod rga_impl_test {
    use super::*;

    #[test]
    fn run() {
        for i in 0..100 {
            crate::test::test::<RgaImpl>(i, 2, 1000);
        }
    }

    #[test]
    fn run3() {
        for seed in 0..100 {
            crate::test::test::<RgaImpl>(seed, 3, 1000);
        }
    }

    #[test]
    fn run_n() {
        for n in 2..10 {
            crate::test::test::<RgaImpl>(123, n, 10000);
        }
    }

    use ctor::ctor;
    #[ctor]
    fn init_color_backtrace() {
        color_backtrace::install();
    }
}
