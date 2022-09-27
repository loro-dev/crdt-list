use std::marker::PhantomData;

use rand::{rngs::StdRng, Rng};

use crate::crdt::Crdt;

#[derive(Debug)]
struct Actor<T: TestFramework> {
    container: T::Container,
    idx: usize,
    ops: Vec<Vec<T::OpUnit>>,
    pending_ops: Vec<T::OpUnit>,
    _phantom: PhantomData<T>,
}

pub trait TestFramework: Crdt {
    fn is_content_eq(a: &Self::Container, b: &Self::Container) -> bool;
    fn new_container(id: usize) -> Self::Container;
    fn new_op(rng: &mut impl Rng, container: &mut Self::Container) -> Self::OpUnit;
}

#[derive(Clone)]
enum Action {
    Sync { from: usize, to: usize },
    NewOp { at: usize },
}

impl<T: TestFramework> Actor<T> {
    fn new(idx: usize, n_container: usize) -> Self {
        Actor {
            container: T::new_container(idx),
            idx,
            ops: vec![Default::default(); n_container],
            pending_ops: Vec::new(),
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
                at: rng.gen_range(0..num_containers),
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

        self.apply_pending();
    }

    fn new_op(&mut self, rng: &mut impl Rng) {
        let value = T::new_op(rng, &mut self.container);
        self.ops[self.idx].push(value.clone());
        T::integrate(&mut self.container, value);
    }

    fn run(containers: &mut [Self], rng: &mut impl Rng, n_actions: usize) {
        for _ in 0..n_actions {
            let action = Self::gen(rng, containers.len());
            match action {
                Action::Sync { from, to } => {
                    let (to_, from_) = arref::array_mut_ref!(containers, [to, from]);
                    to_.sync(from_);
                }
                Action::NewOp { at } => containers[at].new_op(rng),
            }
        }
    }

    fn check(containers: &mut [Self]) {
        for i in 0..(containers.len() - 1) {
            let (a, b) = arref::array_mut_ref!(containers, [i, i + 1]);
            a.sync(b);
            b.sync(a);
            assert!(T::is_content_eq(&a.container, &b.container));
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
