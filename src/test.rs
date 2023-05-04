use std::marker::PhantomData;

use rand::{rngs::StdRng, Rng};

use crate::crdt::ListCrdt;

pub trait TestFramework: ListCrdt {
    type DeleteOp: Clone;
    fn is_content_eq(a: &Self::Container, b: &Self::Container) -> bool;
    fn new_container(id: usize) -> Self::Container;
    /// pos is just a hint, it may not be a valid position
    fn new_op(container: &mut Self::Container, pos: usize) -> Self::OpUnit;

    fn new_del_op(container: &Self::Container, pos: usize, len: usize) -> Self::DeleteOp;
    fn integrate_delete_op(container: &mut Self::Container, op: Self::DeleteOp);
    fn integrate(container: &mut Self::Container, op: Self::OpUnit);
    fn can_integrate(container: &Self::Container, op: &Self::OpUnit) -> bool;
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Action {
    Sync { from: u8, to: u8 },
    NewOp { client_id: u8, pos: u8 },
    Delete { client_id: u8, pos: u8, len: u8 },
}

impl Action {
    pub fn normalize(&mut self, client_len: usize, content_len: usize) {
        let client_len = std::cmp::min(client_len, 255) as u8;
        let content_len = std::cmp::min(content_len, 255) as u8;
        match self {
            Action::Sync { from, to } => {
                *from %= client_len;
                *to %= client_len;
            }
            Action::NewOp { client_id, pos } => {
                *client_id %= client_len;
                *pos %= content_len;
            }
            Action::Delete {
                client_id,
                pos,
                len,
            } => {
                *client_id %= client_len;
                *pos %= content_len;
                *len %= content_len;
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Actor<T: TestFramework> {
    container: T::Container,
    idx: usize,
    ops: Vec<Vec<T::OpUnit>>,
    del_ops: Vec<Vec<T::DeleteOp>>,
    pending_ops: Vec<T::OpUnit>,
    _phantom: PhantomData<T>,
}

impl<T: TestFramework> Actor<T> {
    fn new(idx: u8, n_container: u8) -> Self {
        Actor {
            container: T::new_container(idx as usize),
            idx: idx as usize,
            ops: vec![Default::default(); n_container as usize],
            pending_ops: Vec::new(),
            del_ops: vec![Default::default(); n_container as usize],
            _phantom: PhantomData,
        }
    }

    fn gen(rng: &mut impl Rng, num_containers: usize) -> Action {
        let action = rng.gen_range(0..2);
        match action {
            0 => {
                let from = rng.gen_range(0..num_containers) as u8;
                let to =
                    ((rng.gen_range(1..num_containers) + from as usize) % num_containers) as u8;
                Action::Sync { from, to }
            }
            1 => Action::NewOp {
                client_id: rng.gen_range(0..num_containers) as u8,
                pos: rng.gen_range(0..usize::MAX) as u8,
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
        // repeat the same logic with delete op
        for (op_arr_this, op_arr_other) in self.del_ops.iter_mut().zip(other.del_ops.iter()) {
            if op_arr_this.len() >= op_arr_other.len() {
                continue;
            }

            for op in op_arr_other.iter().skip(op_arr_this.len()) {
                op_arr_this.push(op.clone());
                T::integrate_delete_op(&mut self.container, op.clone());
            }
        }
    }

    fn new_op(&mut self, pos: usize) {
        let value = T::new_op(&mut self.container, pos);
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
            // println!("{:#?}, ", &action); // print actions
            Self::run_action(action, actors);
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

    pub(crate) fn run_action(action: Action, actors: &mut [Actor<T>]) {
        match action {
            Action::Sync { from, to } => {
                let (to_, from_) = arref::array_mut_ref!(actors, [to as usize, from as usize]);
                to_.sync(from_);
            }
            Action::NewOp { client_id: at, pos } => actors[at as usize].new_op(pos as usize),
            Action::Delete {
                client_id,
                pos,
                len,
            } => actors[client_id as usize].new_del_op(pos as usize, len as usize),
        }
    }
}

pub fn test<T: TestFramework>(seed: u64, n_container: usize, round: usize) {
    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(seed);
    let mut containers: Vec<Actor<T>> = Vec::new();
    for i in 0..n_container {
        containers.push(Actor::new(i as u8, n_container as u8));
    }

    Actor::run(&mut containers, &mut rng, round);
    Actor::check(&mut containers);
}

pub fn test_actions<T: TestFramework>(n_container: usize, actions: Vec<Action>) {
    let mut containers: Vec<Actor<T>> = Vec::new();
    for i in 0..n_container {
        containers.push(Actor::new(i as u8, n_container as u8));
    }
    for action in actions {
        Actor::run_action(action, &mut containers);
    }
    Actor::check(&mut containers);
}

pub fn test_with_actions<T: TestFramework>(
    n_container: usize,
    content_len: usize,
    mut actions: Vec<Action>,
) {
    normalize_actions(&mut actions, n_container, content_len);
    let n_container = n_container as u8;
    let mut actors: Vec<Actor<T>> = Vec::new();
    for i in 0..n_container {
        actors.push(Actor::new(i, n_container));
    }

    for action in actions {
        match &action {
            Action::Sync { from, to } => {
                let mut from = *from;
                let to = *to;
                if from == to {
                    from = (from + 1) % n_container;
                }

                let (to_, from_) = arref::array_mut_ref!(&mut actors, [to as usize, from as usize]);
                to_.sync(from_);
            }
            Action::NewOp { client_id: at, pos } => actors[*at as usize].new_op(*pos as usize),
            Action::Delete {
                client_id,
                pos,
                len,
            } => actors[*client_id as usize].new_del_op(*pos as usize, *len as usize),
        }
    }

    Actor::check(&mut actors);
}

pub fn normalize_actions(actions: &mut [Action], n_container: usize, content_len: usize) {
    for action in actions {
        action.normalize(n_container, content_len);
    }
}
