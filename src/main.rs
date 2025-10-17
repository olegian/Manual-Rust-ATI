mod union_find;
use union_find::UnionFind;
use std::{collections::HashMap, sync::{LazyLock, Mutex}};

struct Site {
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

    pub fn observe_var(&mut self, var: String, tag: String) {
        self.observed_var_tags.push((var, tag));
    }

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

    pub fn get_leaders(&self) -> &HashMap<String, String> {
        &self.var_tags
    }
}

static VALUE_UF: LazyLock<Mutex<UnionFind<String>>> = LazyLock::new(|| Mutex::new(UnionFind::new()));
static SITE_UFS: LazyLock<Mutex<Sites>> = LazyLock::new(|| Mutex::new(Sites::new()));

struct Sites {
    locs: HashMap<usize, Site>,
}
impl Sites {
    pub fn new() -> Self {
        Sites { locs: HashMap::new()}
    }

    pub fn get_site(&mut self, id: usize) -> &mut Site {
        if !self.locs.contains_key(&id) {
            self.locs.insert(id, Site::new());
        }

        self.locs.get_mut(&id).unwrap()
    }

    pub fn print_analysis(&self) {
        for (id, site) in self.locs.iter() {
            println!("=== AT SITE {} ===", id);
            for (var, leader) in site.get_leaders() {
                println!("{var} -> {leader}");
            }
        }
    }
}

fn main() {
    simple_func(10, 100);
    simple_func(20, 200);

    // without this line, we should see two abstract type sets in the output
    // due to the conditional on line 118, otherwise all variables will be in the same sets
    simple_func(30, 300); 

    let site_ufs = SITE_UFS.lock().unwrap();
    site_ufs.print_analysis();
}

fn simple_func(x: u32, y: u32) -> u32 {
    let mut value_uf = VALUE_UF.lock().unwrap();
    let mut site_ufs = SITE_UFS.lock().unwrap();
    let site = site_ufs.get_site(0);

    value_uf.make_set(format!("VAL:{}", x));  // for parameter input
    value_uf.make_set(format!("VAL:{}", y));
    site.observe_var("VAR:x".into(), format!("VAL:{}", x));
    site.observe_var("VAR:y".into(), format!("VAL:{}", y));

    let a: u32 = 2;
    value_uf.make_set(format!("LIT:{}", 2));
    value_uf.make_set(format!("VAL:{}", a));
    site.observe_var("VAR:a".into(), format!("VAL:{}", a));

    let b: u32 = 3;
    value_uf.make_set(format!("LIT:{}", 3));
    value_uf.make_set(format!("VAL:{}", b));
    site.observe_var("VAR:b".into(), format!("VAL:{}", b));

    let result = a + x;
    value_uf.make_set(format!("VAL:{}", result));
    value_uf.union(&format!("VAL:{}", result), &format!("VAL:{}", a));
    value_uf.union(&format!("VAL:{}", result), &format!("VAL:{}", x));
    site.observe_var("VAR:result".into(), format!("VAL:{}", result));

    let test = b + y;
    value_uf.make_set(format!("VAL:{}", test));
    value_uf.union(&format!("VAL:{}", test), &format!("VAL:{}", b));
    value_uf.union(&format!("VAL:{}", test), &format!("VAL:{}", y));
    site.observe_var("VAR:test".into(), format!("VAL:{}", test));

    if test > 300 {
        // TODO: THIS IMPLEMENTAITON REQUIRES SSA FORM!
        // is that fine ??? a smarter choice of tag probably addresses this.
        // basically because value_uf has to incorporate both old result here and result2 with tags for proper merging
        let result2 = result + test;
        value_uf.make_set(format!("VAL:{}", result2));
        value_uf.union(&format!("VAL:{}", result2), &format!("VAL:{}", result));
        value_uf.union(&format!("VAL:{}", result2), &format!("VAL:{}", test));
        site.observe_var("VAR:result2".into(), format!("VAL:{}", result2));
    }

    site.update(&mut value_uf);

    return result
}
