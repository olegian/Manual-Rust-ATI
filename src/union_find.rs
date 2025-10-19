use std::collections::HashMap;
use std::hash::Hash;

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
pub struct UnionFind<SetId>
where
    SetId: Eq + Hash + Clone,
{
    id_to_index: HashMap<SetId, usize>,
    index_to_set: Vec<SetId>,
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl<SetId> UnionFind<SetId>
where
    SetId: Eq + Hash + Clone,
{
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
    pub fn make_set(&mut self, id: SetId) -> Option<usize> {
        if self.id_to_index.contains_key(&id) {
            return Some(*self.id_to_index.get(&id).unwrap());
        }

        let index = self.parent.len();
        self.id_to_index.insert(id.clone(), index);
        self.index_to_set.push(id);
        self.parent.push(index);
        self.rank.push(0);
        
        None
    }

    fn get_index(&self, id: &SetId) -> Option<usize> {
        self.id_to_index.get(id).copied()
    }

    /// Find the leader SetId which represents the set that
    /// the passed in SetId identifies.
    pub fn find(&mut self, id: &SetId) -> Option<SetId> {
        let index = self.get_index(id)?;
        let leader_index = self.find_index(index);
        Some(self.index_to_set[leader_index].clone())
    }

    /// Merges the sets which the two passed in id's identify.
    /// Returns the leader SetId of the merged set.
    pub fn union(&mut self, id1: &SetId, id2: &SetId) -> Option<SetId> {
        let i1 = self.get_index(id1)?;
        let i2 = self.get_index(id2)?;
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
