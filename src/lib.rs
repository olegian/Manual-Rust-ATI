extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitInt, parse_macro_input, parse::{Parse}};

#[proc_macro_attribute]
pub fn ati_site(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    // let site_id: Result<usize, syn::Error> = parse_macro_input!(attr as LitInt).base10_parse();
    // let site_id = site_id.unwrap();
    // let site_id = proc_macro2::TokenStream::from(attr) as LitInt;

    let prelude = quote! {
        // let mut value_uf = VALUE_UF.lock().unwrap();
        // let mut site_ufs = SITE_UFS.lock().unwrap();
        // let site = site_ufs.get_site(0);  // create a new analysis site. View site.rs for more info
        // println!("{:?}", "hello");
        println!("HELLO WORLD");
        // let mut value_uf = VALUE_UF.lock().unwrap();
    };

    input_fn
        .block
        .stmts
        .insert(0, syn::parse2(prelude).unwrap());

    quote! { #input_fn }.into()
}

/// Declares all necessary structs for dynamic ATI
/// Defines global statics 
#[proc_macro_attribute]
pub fn ati_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // TODO: this is a really ugly way, but the imports werent resolving the 
    // way I expected them too, so for now, I just dumped everything in here.
    let added_item = quote! {
        use std::{sync::{LazyLock, Mutex}};
        use std::collections::HashMap;

        #[derive(PartialEq, Eq, Hash, Clone, Debug)]
        pub struct Tag {
            addr: String,
        }

        impl Tag {
            pub fn new<T>(value: &T) -> Self {
                Tag {
                    addr: format!("{:p}", value),
                }
            }
        }

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
            pub fn observe_var<V>(&mut self, name: &str, var: &V) {
                self.observed_var_tags.push((name.into(), Tag::new(var)));
            }

            /// Algorithm from "Dynamic inference of Abstract Types" by Guo et. al.
            pub fn update(&mut self, value_uf: &mut UnionFind) {
                for (new_var, new_var_tag) in &self.observed_var_tags {
                    let new_leader_tag = value_uf.find(new_var_tag).unwrap(); // ? is this unwrap safe? 
                    let new_leader_tag = self.type_uf.introduce_tag(new_leader_tag);

                    if let Some(old_tag) = self.var_tags.get(new_var) {
                        let old_leader_tag = value_uf.find(old_tag).unwrap();

                        let merged = self.type_uf.union_tags(&old_leader_tag, &new_leader_tag).unwrap();
                        self.var_tags.insert(new_var.clone(), merged);
                    } else {
                        self.var_tags.insert(new_var.clone(), new_leader_tag);
                    }
                }
            }

            /// Returns the mapping of the ATI output, var identifiers
            /// to value interaction set leader tags.
            pub fn get_leaders(&self) -> &HashMap<String, Tag> {
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
                        println!("{var} -> {leader:?}");
                    }
                }
            }
        }

        /// Implementation of a UnionFind data structure, in which elements are identified via
        /// a unique SetId (which necessarily implements `Eq + Hash + Clone`). This allows
        /// SetId to be a String representation of the address of a particular variable,
        /// any other identifying information, or even a full struct which stores this identifier
        /// alongside whatever useful metadata is helpful for debugging or organizational 
        /// purposes.
        /// 
        /// Each inserted element maintains a 1-1 mapping with it's SetId, passed in when
        /// invoking `make_set`. Each element tracks it's parent via the `parent` Vec.
        /// When elements are added into the structure, it appends a new element to this
        /// Vec. `parent[i]` is the index of the leader element. If `parent[i] == i`, 
        /// then element `i` is the leader. `index_to_set[i]` returns the SetId (including
        /// whatever metadata was associated with it). `find(SetId)` will locate the SetId
        /// of the set leader.
        /// 
        /// `rank` is used for determining which direction to perform the union, ultimately
        /// just the standard optimization done with UnionFind structures.
        pub struct UnionFind {
            id_to_index: HashMap<Tag, usize>,
            pub index_to_set: Vec<Tag>,
            parent: Vec<usize>,
            rank: Vec<usize>,
        }

        impl UnionFind { 
            /// Creates a new UnionFind
            pub fn new() -> Self {
                Self {
                    id_to_index: HashMap::new(),
                    index_to_set: Vec::new(),
                    parent: Vec::new(),
                    rank: Vec::new(),
                }
            }

            /// Creates a new unique element in its own set, to be tracked 
            /// within this UnionFind. Duplicate SetIds are disallowed.
            /// 
            /// Returns Some(i) if this SetId already corresponds to some set
            /// at parent[i] with rank[i]. Returns None if this operation created
            /// a new set.
            pub fn make_set<V>(&mut self, var: &V) -> Tag  {
                let id = Tag::new(var);
                self.introduce_tag(id)
            }

            /// Similar to make_set, but does not create a new tag out of a variable
            /// just accepts an existing tag as input
            pub fn introduce_tag(&mut self, id: Tag) -> Tag {
                if self.id_to_index.contains_key(&id) {
                    // return Some(*self.id_to_index.get(&id).unwrap());
                    return id;
                }

                let index = self.parent.len();
                self.id_to_index.insert(id.clone(), index);
                self.index_to_set.push(id.clone());
                self.parent.push(index);
                self.rank.push(0);

                return id;
            }

            fn get_index(&self, id: &Tag) -> Option<usize> {
                self.id_to_index.get(id).copied()
            }

            /// Find the leader SetId which represents the set that
            /// the passed in SetId identifies.
            pub fn find(&mut self, tag: &Tag) -> Option<Tag> {
                let index = self.get_index(tag)?;
                let leader_index = self.find_index(index);
                Some(self.index_to_set[leader_index].clone())
            }

            /// Merges the sets which the two passed in id's identify.
            /// Returns the leader SetId of the merged set.
            pub fn union_vals<V>(&mut self, v1: &V, v2: &V) -> Option<Tag> {
                let id1 = Tag::new(v1);
                let id2 = Tag::new(v2);
                self.union_tags(&id1, &id2)
            }

            pub fn union_tags(&mut self, t1: &Tag, t2: &Tag) -> Option<Tag> {
                let i1 = self.get_index(t1)?;
                let i2 = self.get_index(t2)?;
                let leader_index = self.union_indices(i1, i2);
                Some(self.index_to_set[leader_index].clone())
            }

            /// Internal find function w/ path compression
            fn find_index(&mut self, x: usize) -> usize {
                if self.parent[x] != x {
                    self.parent[x] = self.find_index(self.parent[x]);
                }
                self.parent[x]
            }

            /// Internal union, performing union by rank
            fn union_indices(&mut self, x: usize, y: usize) -> usize {
                let x_root = self.find_index(x);
                let y_root = self.find_index(y);

                if x_root == y_root {
                    return x_root;
                }

                // Union towards larger rank
                if self.rank[x_root] < self.rank[y_root] {
                    self.parent[x_root] = y_root;
                    y_root
                } else if self.rank[x_root] > self.rank[y_root] {
                    self.parent[y_root] = x_root;
                    x_root
                } else {
                    self.parent[y_root] = x_root;
                    self.rank[x_root] += 1;
                    x_root
                }
            }
        }

        static VALUE_UF: LazyLock<Mutex<UnionFind>> = LazyLock::new(|| Mutex::new(UnionFind::new()));
        static SITE_UFS: LazyLock<Mutex<Sites>> = LazyLock::new(|| Mutex::new(Sites::new()));
    };

    let expanded = quote! {
        #input_fn
        #added_item
    };

    expanded.into()
}