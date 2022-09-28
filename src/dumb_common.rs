use std::{collections::HashSet, ops::Deref};

use crate::crdt::OpSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Op {
    pub id: OpId,
    pub left: Option<OpId>,
    pub right: Option<OpId>,
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

#[derive(Debug, Default)]
pub struct Container {
    pub content: Vec<Op>,
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
}

pub struct Cursor<'a> {
    pub arr: &'a mut Vec<Op>,
    pub pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Cursor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
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
        }

        if Some(op.id) == self.start {
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

impl Deref for Cursor<'_> {
    type Target = Op;

    fn deref(&self) -> &Self::Target {
        &self.arr[self.pos]
    }
}
