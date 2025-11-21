use std::collections::HashMap;

use crate::Tag;
use crate::union_find::UnionFind;

/// A site captures a set of lines in the source code under analysis. A site starts
/// when it is created with `new()`, collects variables under analysis at that site
/// with `observe_var()`, and is then closed with `update()`.
///
/// During execution, when new variables are binded via `let`, the name of the variable, and
/// the tag of the value stored in that variable is loaded into `observed_var_tags`.
/// The tag of the value must match the tag stored in the global interaction set.
///
/// Then, when `update` is called, the observed variables are added into `type_uf`,
/// using the passed `value_uf` state (which tracks which value tags have been
/// placed into the same interaction set, globally) to determine which variables
/// belong to the same abstract types.
///
/// Two variables are considered to have the same abstract type, if there exists some
/// execution path in which the tags of the values binded to those variables have at some point
/// interacted, over the course of the entire programs execution.
///
/// `var_tags` contains the ATI output, mapping the variable identifiers (names) to a value tag,
/// the leader tag of a set of values in `value_uf` which have been observed interacting together.
pub struct Site {
    type_uf: UnionFind,
    var_tags: HashMap<String, Tag>,
    observed_var_tags: Vec<(String, Tag)>,
    name: String, // Debug information
}

impl Site {
    pub fn new(name: &str) -> Self {
        Site {
            type_uf: UnionFind::new(),
            var_tags: HashMap::new(),
            observed_var_tags: Vec::new(),
            name: name.to_owned(),
        }
    }

    /// Registers a new variable pertaining to this analysis site.
    pub fn observe_var(&mut self, name: &str, var_tag: &Tag) {
        self.observed_var_tags.push((name.into(), var_tag.clone()));
    }

    /// Algorithm from "Dynamic inference of Abstract Types" by Guo et. al.
    pub fn update(&mut self, value_uf: &mut UnionFind) {
        for (new_var, new_var_tag) in &self.observed_var_tags {
            let new_leader_tag = value_uf.find(new_var_tag).unwrap(); // ? is this unwrap safe? 
            let new_leader_tag = self.type_uf.introduce_tag(new_leader_tag);

            if let Some(old_tag) = self.var_tags.get(new_var) {
                let old_leader_tag = value_uf.find(old_tag).unwrap();

                let merged = self
                    .type_uf
                    .union_tags(&old_leader_tag, &new_leader_tag)
                    .unwrap();
                self.var_tags.insert(new_var.clone(), merged);
            } else {
                self.var_tags.insert(new_var.clone(), new_leader_tag);
            }
        }
    }

    pub fn report(&self) {
        println!("=== {} === ", self.name);
        for (var, tag) in self.var_tags.iter() {
            println!("{var} -> {tag:?}");
        }
        println!("\n");
    }
}

pub struct Sites {
    locs: HashMap<String, Site>,
}
impl Sites {
    pub fn new() -> Self {
        Sites {
            locs: HashMap::new(),
        }
    }

    /// Registers a new site with a given id, or returns
    /// the site with the provided id.
    pub fn extract(&mut self, id: &str) -> Site {
        if !self.locs.contains_key(id) {
            Site::new(id)
        } else {
            self.locs.remove(id).unwrap()
        }
    }

    pub fn stash(&mut self, site: Site) {
        self.locs.insert(site.name.clone(), site);
    }

    pub fn report(&self) {
        for (_, site) in self.locs.iter() {
            site.report();
        }
    }
}
