use std::collections::HashMap;

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
    type_uf: UnionFind<String>,
    var_tags: HashMap<String, String>,
    observed_var_tags: Vec<(String, String)>,
}

impl Site {
    pub fn new() -> Self {
        Site {
            type_uf: UnionFind::new(),
            var_tags: HashMap::new(),
            observed_var_tags: Vec::new(),
        }
    }

    /// Registers a new variable pertaining to this analysis site.
    pub fn observe_var(&mut self, var: String, tag: String) {
        self.observed_var_tags.push((var, tag));
    }

    /// Algorithm from "Dynamic inference of Abstract Types" by Guo et. al.
    pub fn update(&mut self, value_uf: &mut UnionFind<String>) {
        for (new_var, new_var_tag) in &self.observed_var_tags {
            let new_leader_tag = value_uf.find(new_var_tag).unwrap(); // ? is this unwrap safe? 
            self.type_uf.make_set(new_leader_tag.clone());

            if let Some(old_tag) = self.var_tags.get(new_var) {
                let old_leader_tag = value_uf.find(old_tag).unwrap();

                let merged = self.type_uf.union(&old_leader_tag, &new_leader_tag).unwrap();
                self.var_tags.insert(new_var.clone(), merged);
            } else {
                self.var_tags.insert(new_var.clone(), new_leader_tag);
            }
        }
    }

    /// Returns the mapping of the ATI output, var identifiers
    /// to value interaction set leader tags.
    pub fn get_leaders(&self) -> &HashMap<String, String> {
        &self.var_tags
    }
}

pub struct Sites {
    locs: HashMap<usize, Site>,
}
impl Sites {
    pub fn new() -> Self {
        Sites { locs: HashMap::new()}
    }

    /// Registers a new site with a given id, or returns
    /// the site with the provided id.
    pub fn get_site(&mut self, id: usize) -> &mut Site {
        if !self.locs.contains_key(&id) {
            self.locs.insert(id, Site::new());
        }

        self.locs.get_mut(&id).unwrap()
    }

    /// Simple function to print the output of all registered sites.
    pub fn print_analysis(&self) {
        for (id, site) in self.locs.iter() {
            println!("=== AT SITE {} ===", id);
            for (var, leader) in site.get_leaders() {
                println!("{var} -> {leader}");
            }
        }
    }
}
