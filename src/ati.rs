use std::{sync::{Arc, LazyLock, Mutex}};

use crate::{
    site::{Site, Sites},
    tag::TaggedValue,
    union_find::UnionFind,
};


pub struct ATI {
    value_uf: UnionFind,
    sites: Sites,
}

impl ATI {
    pub fn new() -> Self {
        Self {
            value_uf: UnionFind::new(),
            sites: Sites::new(),
        }
    }

    // use this function whenever a new literal is created
    pub fn track<T>(
        value: T, // value of variable
    ) -> TaggedValue<T>
    where
        T: Copy,
    {
        let id = ATI_ANALYSIS.lock().unwrap().value_uf.make_set();
        TaggedValue::new(value, id)
    }

    pub fn get_site(&mut self, id: &str) -> Site {
        self.sites.extract(id)
    }

    pub fn update_site(&mut self, mut site: Site) {
        site.update(&mut self.value_uf);
        self.sites.stash(site);
    }

    pub fn union_tags<T>(&mut self, tv1: &TaggedValue<T>, tv2: &TaggedValue<T>)
    where
        T: Copy,
    {
        self.value_uf.union_tags(&tv1.1, &tv2.1);
    }

    pub fn report(&self) {
        self.sites.report();
    }
}
