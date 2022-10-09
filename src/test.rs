use std::marker::PhantomData;

use rand::{rngs::StdRng, Rng};

use crate::crdt::ListCrdt;

pub trait TestFramework: ListCrdt {
    type DeleteOp: Clone;
    fn is_content_eq(a: &Self::Container, b: &Self::Container) -> bool;
    fn new_container(id: usize) -> Self::Container;
    /// pos is just a hint, it may not be a valid position
    fn new_op(rng: &mut impl Rng, container: &mut Self::Container, pos: usize) -> Self::OpUnit;

    fn new_del_op(container: &Self::Container, pos: usize, len: usize) -> Self::DeleteOp;
    fn integrate_delete_op(container: &mut Self::Container, op: Self::DeleteOp);
    fn can_apply_del_op(container: &Self::Container, op: &Self::DeleteOp) -> bool;
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Action {
    Sync {
        from: usize,
        to: usize,
    },
    NewOp {
        client_id: usize,
        pos: usize,
    },
    Delete {
        client_id: usize,
        pos: usize,
        len: usize,
    },
}

#[derive(Debug)]
struct Actor<T: TestFramework> {
    container: T::Container,
    idx: usize,
    ops: Vec<Vec<T::OpUnit>>,
    del_ops: Vec<Vec<T::DeleteOp>>,
    pending_ops: Vec<T::OpUnit>,
    pending_del_ops: Vec<T::DeleteOp>,
    _phantom: PhantomData<T>,
}

impl<T: TestFramework> Actor<T> {
    fn new(idx: usize, n_container: usize) -> Self {
        Actor {
            container: T::new_container(idx),
            idx,
            ops: vec![Default::default(); n_container],
            pending_ops: Vec::new(),
            del_ops: vec![Default::default(); n_container],
            pending_del_ops: Vec::new(),
            _phantom: PhantomData,
        }
    }

    fn gen(rng: &mut impl Rng, num_containers: usize) -> Action {
        let action = rng.gen_range(0..2);
        match action {
            0 => {
                let from = rng.gen_range(0..num_containers);
                let to = (rng.gen_range(1..num_containers) + from) % num_containers;
                Action::Sync { from, to }
            }
            1 => Action::NewOp {
                client_id: rng.gen_range(0..num_containers),
                pos: rng.gen_range(0..usize::MAX),
            },
            _ => unreachable!(),
        }
    }

    fn apply_pending(&mut self) {
        let mut pending = std::mem::take(&mut self.pending_ops);
        while !pending.is_empty() {
            let current = std::mem::take(&mut pending);
            for op in current {
                if T::can_integrate(&self.container, &op) {
                    T::integrate(&mut self.container, op.clone());
                } else {
                    pending.push(op.clone());
                }
            }
        }

        // repeat the same logic with delete op
        let mut pending = std::mem::take(&mut self.pending_del_ops);
        while !pending.is_empty() {
            let current = std::mem::take(&mut pending);
            for op in current {
                if T::can_apply_del_op(&self.container, &op) {
                    T::integrate_delete_op(&mut self.container, op.clone());
                } else {
                    pending.push(op.clone());
                }
            }
        }
    }

    fn sync(&mut self, other: &Self) {
        for (op_arr_this, op_arr_other) in self.ops.iter_mut().zip(other.ops.iter()) {
            if op_arr_this.len() >= op_arr_other.len() {
                continue;
            }

            for op in op_arr_other.iter().skip(op_arr_this.len()) {
                op_arr_this.push(op.clone());
                if T::can_integrate(&self.container, op) {
                    T::integrate(&mut self.container, op.clone());
                } else {
                    self.pending_ops.push(op.clone());
                }
            }
        }

        // repeat the same logic with delete op
        for (op_arr_this, op_arr_other) in self.del_ops.iter_mut().zip(other.del_ops.iter()) {
            if op_arr_this.len() >= op_arr_other.len() {
                continue;
            }

            for op in op_arr_other.iter().skip(op_arr_this.len()) {
                op_arr_this.push(op.clone());
                if T::can_apply_del_op(&self.container, op) {
                    T::integrate_delete_op(&mut self.container, op.clone());
                } else {
                    self.pending_del_ops.push(op.clone());
                }
            }
        }

        self.apply_pending();
    }

    fn new_op(&mut self, rng: &mut impl Rng, pos: usize) {
        let value = T::new_op(rng, &mut self.container, pos);
        self.ops[self.idx].push(value.clone());
        T::integrate(&mut self.container, value);
    }

    fn new_del_op(&mut self, pos: usize, len: usize) {
        let value = T::new_del_op(&self.container, pos, len);
        self.del_ops[self.idx].push(value.clone());
        T::integrate_delete_op(&mut self.container, value);
    }

    fn run(actors: &mut [Self], rng: &mut impl Rng, n_actions: usize) {
        for _ in 0..n_actions {
            let action = Self::gen(rng, actors.len());
            match action {
                Action::Sync { from, to } => {
                    let (to_, from_) = arref::array_mut_ref!(actors, [to, from]);
                    to_.sync(from_);
                }
                Action::NewOp { client_id: at, pos } => actors[at].new_op(rng, pos),
                Action::Delete {
                    client_id,
                    pos,
                    len,
                } => actors[client_id].new_del_op(pos, len),
            }
        }
    }

    fn check(containers: &mut [Self]) {
        for i in 0..(containers.len() - 1) {
            let (a, b) = arref::array_mut_ref!(containers, [i, i + 1]);
            a.sync(b);
            b.sync(a);
            let eq = T::is_content_eq(&a.container, &b.container);
            if !eq {
                dbg!(&a.container);
                dbg!(&b.container);
                panic!("Containers are not equal");
            }
        }
    }
}

pub fn test<T: TestFramework>(seed: u64, n_container: usize, round: usize) {
    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(seed);
    let mut containers: Vec<Actor<T>> = Vec::new();
    for i in 0..n_container {
        containers.push(Actor::new(i, n_container));
    }

    Actor::run(&mut containers, &mut rng, round);
    Actor::check(&mut containers);
}

pub fn test_with_actions<T: TestFramework>(n_container: usize, actions: &[Action]) {
    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(123);
    let mut actors: Vec<Actor<T>> = Vec::new();
    for i in 0..n_container {
        actors.push(Actor::new(i, n_container));
    }

    for action in actions {
        match action {
            Action::Sync { from, to } => {
                let to = *to % n_container;
                let mut from = *from % n_container;
                if from == to {
                    from = (from + 1) % n_container;
                }

                let (to_, from_) = arref::array_mut_ref!(&mut actors, [to, from]);
                to_.sync(from_);
            }
            Action::NewOp { client_id: at, pos } => {
                actors[*at % n_container].new_op(&mut rng, *pos)
            }
            Action::Delete {
                client_id,
                pos,
                len,
            } => actors[*client_id % n_container].new_del_op(*pos, *len % 5),
        }
    }

    Actor::check(&mut actors);
}
