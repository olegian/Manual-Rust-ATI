use crate::{
    site::{Site, Sites},
    tag::Tag,
    union_find::UnionFind,
};

pub struct ATI {
    value_uf: UnionFind,
    sites: Sites,
}

impl ATI {
    pub fn new() -> Self {
        ATI {
            value_uf: UnionFind::new(),
            sites: Sites::new(),
        }
    }

    pub fn untracked<V>(&mut self, v: &V) -> Tag {
        self.value_uf.make_set(v)
    }

    pub fn tracked<V>(&mut self, var_name: &str, v: &V, site: &mut Site) -> Tag {
        let tag = self.value_uf.make_set(v);
        site.observe_var(var_name, &tag);
        tag
    }

    pub fn get_site(&mut self, id: &str) -> Site {
        self.sites.extract(id)
    }

    pub fn update_site(&mut self, mut site: Site) {
        site.update(&mut self.value_uf);
        self.sites.stash(site);
    }

    pub fn union_tags(&mut self, tags: &[&Tag]) {
        for tags in tags.windows(2) {
            self.value_uf.union_tags(tags[0], tags[1]);
        }
    }

    pub fn report(&self) {
        self.sites.report();
    }
}
