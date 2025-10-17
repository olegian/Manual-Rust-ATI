use std::collections::HashMap;
use std::hash::Hash;

pub struct UnionFind<Tag>
where
    Tag: Eq + Hash + Clone,
{
    tag_to_index: HashMap<Tag, usize>,
    index_to_tag: Vec<Tag>,
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl<Tag> UnionFind<Tag>
where
    Tag: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            tag_to_index: HashMap::new(),
            index_to_tag: Vec::new(),
            parent: Vec::new(),
            rank: Vec::new(),
        }
    }

    /// Returns Some(idx) if this tag already corresponds to some set
    /// at parent[i] with rank[i]. Returns None if this a new set
    pub fn make_set(&mut self, tag: Tag) -> Option<usize> {
        if self.tag_to_index.contains_key(&tag) {
            return Some(*self.tag_to_index.get(&tag).unwrap());
        }

        let index = self.parent.len();
        self.tag_to_index.insert(tag.clone(), index);
        self.index_to_tag.push(tag);
        self.parent.push(index);
        self.rank.push(0);
        
        None
    }

    fn get_index(&self, tag: &Tag) -> Option<usize> {
        self.tag_to_index.get(tag).copied()
    }

    /// Find the leader tag for the set containing `tag`.
    pub fn find(&mut self, tag: &Tag) -> Option<Tag> {
        let index = self.get_index(tag)?;
        let leader_index = self.find_index(index);
        Some(self.index_to_tag[leader_index].clone())
    }

    /// Returns the leader tag of the merged set.
    pub fn union(&mut self, tag1: &Tag, tag2: &Tag) -> Option<Tag> {
        let i1 = self.get_index(tag1)?;
        let i2 = self.get_index(tag2)?;
        let leader_index = self.union_indices(i1, i2);
        Some(self.index_to_tag[leader_index].clone())
    }

    /// w/ path compression
    fn find_index(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find_index(self.parent[x]);
        }
        self.parent[x]
    }

    /// union by rank
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
