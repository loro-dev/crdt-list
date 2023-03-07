use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use crate::crdt::{GetOp, OpSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Op {
    pub id: OpId,
    pub lamport: u32,
    pub left: Option<OpId>,
    pub right: Option<OpId>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpId {
    pub client_id: usize,
    pub clock: usize,
}

#[derive(Default)]
pub struct OpSetImpl {
    pub set: HashSet<OpId>,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Content(Vec<Op>);

impl Deref for Content {
    type Target = Vec<Op>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Content {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Content {
    pub fn real_len(&self) -> usize {
        self.0.iter().filter(|x| !x.deleted).count()
    }

    pub fn real_index(&self, index: usize) -> usize {
        let mut real_index = 0;
        for i in 0..self.0.len() {
            if !self.0[i].deleted {
                if real_index == index {
                    return i;
                }
                real_index += 1;
            }
        }
        panic!("index out of range");
    }

    pub fn iter_real(&self) -> impl Iterator<Item = &Op> {
        self.0.iter().filter(|x| !x.deleted)
    }

    pub fn iter_real_mut(&mut self) -> impl Iterator<Item = &mut Op> {
        self.0.iter_mut().filter(|x| !x.deleted)
    }
}

#[derive(Debug, Default)]
pub struct Container {
    pub content: Content,
    /// exclusive end
    pub version_vector: Vec<usize>,
    pub max_clock: usize,
    pub id: usize,
}

pub struct Iter<'a> {
    pub arr: &'a mut Vec<Op>,
    pub index: usize,
    pub start: Option<OpId>,
    pub end: Option<OpId>,
    pub done: bool,
    pub started: bool,
    pub exclude_end: bool,
}

pub struct Cursor<'a> {
    pub arr: &'a mut Vec<Op>,
    pub pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Cursor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.start.is_none() {
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

            if Some(op.id) == self.end {
                self.done = true;
                if self.exclude_end {
                    return None;
                }
            }

            if Some(op.id) == self.start {
                self.started = true;
                if self.exclude_end {
                    continue;
                }
            }

            if !self.started {
                continue;
            }

            break;
        }

        Some(Cursor {
            arr: unsafe { &mut *(self.arr as *mut _) },
            pos: self.index - 1,
        })
    }
}

impl<'a> GetOp for Cursor<'a> {
    type Target = Op;

    fn get_op(&self) -> Self::Target {
        self.arr[self.pos].clone()
    }
}
