use std::{collections::HashSet, ops::Deref};

use crate::{
    crdt::{ListCrdt, OpSet},
    test::TestFramework,
    yata,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Op {
    id: OpId,
    left: OpId,
    right: OpId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpId {
    client_id: usize,
    clock: usize,
}

const START_OP_ID: OpId = OpId {
    client_id: usize::MAX,
    clock: 0,
};
const END_OP_ID: OpId = OpId {
    client_id: usize::MAX,
    clock: 99,
};

#[derive(Default)]
pub struct OpSetImpl {
    set: HashSet<OpId>,
}

impl OpSet<Op, OpId> for OpSetImpl {
    fn insert(&mut self, value: &Op) {
        self.set.insert(value.id);
    }

    fn contain(&self, id: OpId) -> bool {
        self.set.contains(&id)
    }

    fn clear(&mut self) {
        self.set.clear();
    }
}

#[derive(Debug, Default)]
pub struct Container {
    content: Vec<Op>,
    /// exclusive end
    version_vector: Vec<usize>,
    max_clock: usize,
    id: usize,
}

pub struct Iter<'a> {
    arr: &'a mut Vec<Op>,
    index: usize,
    start: OpId,
    end: OpId,
    done: bool,
    started: bool,
}

pub struct Cursor<'a> {
    arr: &'a mut Vec<Op>,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Cursor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == START_OP_ID {
            self.started = true;
        }

        if self.done {
            return None;
        }

        if self.index >= self.arr.len() {
            return None;
        }

        let op = &self.arr[self.index];
        self.index += 1;

        if op.id == self.end {
            self.done = true;
        }

        if op.id == self.start {
            self.started = true;
        }

        if !self.started {
            return self.next();
        }

        Some(Cursor {
            arr: unsafe { &mut *(self.arr as *mut _) },
            pos: self.index - 1,
        })
    }
}

impl YataImpl {
    fn container_contains(
        container: &<Self as ListCrdt>::Container,
        op_id: <Self as ListCrdt>::OpId,
    ) -> bool {
        if op_id == START_OP_ID || op_id == END_OP_ID {
            return true;
        }
        container.content.iter().any(|x| x.id == op_id)

        // if container.version_vector.len() <= op_id.client_id {
        //     return false;
        // }

        // container.version_vector[op_id.client_id] > op_id.clock
    }
}

impl Deref for Cursor<'_> {
    type Target = Op;

    fn deref(&self) -> &Self::Target {
        &self.arr[self.pos]
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
        from: Self::OpId,
        to: Self::OpId,
    ) -> Self::Iterator<'_> {
        Iter {
            arr: &mut container.content,
            index: 0,
            start: from,
            end: to,
            done: false,
            started: false,
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
        yata::integrate::<YataImpl>(container, op);

        container.version_vector[id.client_id] = id.clock + 1;
    }

    fn can_integrate(container: &Self::Container, op: &Self::OpUnit) -> bool {
        Self::container_contains(container, op.left)
            && Self::container_contains(container, op.right)
            && (op.id.clock == 0
                || Self::container_contains(
                    container,
                    OpId {
                        client_id: op.id.client_id,
                        clock: op.id.clock - 1,
                    },
                ))
    }
}

impl yata::Yata for YataImpl {
    fn left_origin(op: &Self::OpUnit) -> Self::OpId {
        op.left
    }

    fn right_origin(op: &Self::OpUnit) -> Self::OpId {
        op.right
    }

    fn insert_after(anchor: &mut Self::Cursor<'_>, op: Self::OpUnit) {
        if anchor.pos + 1 >= anchor.arr.len() {
            anchor.arr.push(op);
        } else {
            anchor.arr.insert(anchor.pos + 1, op);
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
            (START_OP_ID, END_OP_ID)
        } else if insert_pos == 0 {
            (START_OP_ID, container.content[0].id)
        } else if insert_pos == container.content.len() {
            (container.content[insert_pos - 1].id, END_OP_ID)
        } else {
            (
                container.content[insert_pos - 1].id,
                container.content[insert_pos].id,
            )
        };

        let ans = Op {
            id: OpId {
                client_id: container.id,
                clock: container.max_clock,
            },
            left,
            right,
        };

        container.max_clock += 1;
        ans
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
    fn run3() {
        for seed in 0..100 {
            crate::test::test::<YataImpl>(seed, 3, 1000);
        }
    }

    #[test]
    fn run_n() {
        for n in 2..10 {
            crate::test::test::<YataImpl>(123, n, 10000);
        }
    }

    use ctor::ctor;
    #[ctor]
    fn init_color_backtrace() {
        color_backtrace::install();
    }
}
