use std::{ops::{Add, Div, Mul, Sub}};

use crate::ati::{ATI, ATI_ANALYSIS};

pub type Id = u64;

/// Generates unique increasing integer IDs for use as tags
pub struct Tagger {
    next_id: Id,
}

impl Tagger {
    pub fn new() -> Self {
        Tagger { next_id: 0 }
    }

    pub fn tag(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;

        id
    }
}


#[derive(Clone, Copy)]
pub struct TaggedValue<T: Copy>(pub T, pub Id);

impl<T> TaggedValue<T>
where
    T: Copy,
{
    pub fn new(value: T, id: Id) -> Self {
        Self (value, id)
    }

    pub fn unbind(&self) -> T {
        self.0
    }
}

// MARK: Ops + - * /
// restrict T to primative types, but also that doesnt exist lol in rust
impl<T> Add<TaggedValue<T>> for TaggedValue<T> 
where T: Add<Output = T> + Copy {
    type Output = TaggedValue<T>;

    fn add(self, rhs: TaggedValue<T>) -> Self::Output {
        let res = ATI::track(self.0 + rhs.0);

        let mut ati = ATI_ANALYSIS.lock().unwrap();
        ati.union_tags(&self, &rhs);
        ati.union_tags(&res, &self);

        res
    }
}

impl<T> Sub<TaggedValue<T>> for TaggedValue<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = TaggedValue<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        let res = ATI::track(self.0 - rhs.0);

        let mut ati = ATI_ANALYSIS.lock().unwrap();
        ati.union_tags(&self, &rhs);
        ati.union_tags(&res, &self);

        res
    }
}

impl<T> Mul<TaggedValue<T>> for TaggedValue<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = TaggedValue<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        let res = ATI::track(self.0 * rhs.0);

        let mut ati = ATI_ANALYSIS.lock().unwrap();
        ati.union_tags(&self, &rhs);
        ati.union_tags(&res, &self);

        res
    }
}

impl<T> Div<TaggedValue<T>> for TaggedValue<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = TaggedValue<T>;

    fn div(self, rhs: Self) -> Self::Output {
        let res = ATI::track(self.0 / rhs.0);

        let mut ati = ATI_ANALYSIS.lock().unwrap();
        ati.union_tags(&self, &rhs);
        ati.union_tags(&res, &self);

        res
    }
}

// MARK: EQUALITY AND ORDERING
impl<T> PartialEq for TaggedValue<T>
where
    T: Copy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        ATI_ANALYSIS.lock().unwrap().union_tags(&self, &other);
        self.0 == other.0
    }
}
impl<T> Eq for TaggedValue<T> where T: Copy + PartialEq {}

impl<T> PartialOrd for TaggedValue<T>
where
    T: Copy + PartialEq + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(core::cmp::Ordering::Equal) => Some(core::cmp::Ordering::Equal),
            ord => return ord,
        }
    }
}

impl<T> Ord for TaggedValue<T>
where
    T: Copy + PartialEq + PartialOrd,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        ATI_ANALYSIS.lock().unwrap().union_tags(&self, other);
        self.0.partial_cmp(&other.0).unwrap()
    }
}
