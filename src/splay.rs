use std::cmp::Ordering::{Equal, Greater, Less};

type Idx = usize;

#[derive(Clone, Copy, Debug, PartialEq)]
struct OptionIdx(Idx);
const IDX_NONE: OptionIdx = OptionIdx(Idx::MAX);

impl OptionIdx {
    #[inline]
    fn to_option(self) -> Option<Idx> {
        if self == IDX_NONE {
            None
        } else {
            Some(self.0)
        }
    }
}

struct Node<K, V> {
    key: K,
    value: V,
    left: OptionIdx,
    right: OptionIdx,
}

pub struct Splay<K, V> {
    root: OptionIdx,
    nodes: Vec<Node<K, V>>,
}

pub struct SplayIter<'a, K, V> {
    tree: &'a Splay<K, V>,
    path: Vec<(Idx, bool)>,
}

impl<'a, K: Ord, V> SplayIter<'a, K, V> {
    fn new(tree: &'a Splay<K, V>) -> Self {
        let path = Vec::new();
        let mut t = SplayIter { tree, path };
        if let Some(root) = tree.root.to_option() {
            t.towards_min(root);
        }
        t
    }

    fn towards_min(&mut self, idx: Idx) {
        let mut idx = Some(idx);

        while let Some(i) = idx {
            self.path.push((i, false));
            idx = self.tree.nodes[i].left.to_option();
        }
    }

    fn upwards(&mut self) {
        while let Some((_, right_subtree_visited)) = self.path.last() {
            if !right_subtree_visited {
                break;
            }
            self.path.pop();
        }
    }
}

impl<'a, K: Ord, V> Iterator for SplayIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let (node_idx, visited_right_subtree) = self.path.last_mut()?;
        let node = &self.tree.nodes[*node_idx];

        match (&visited_right_subtree, node.right.to_option()) {
            (false, Some(right)) => {
                *visited_right_subtree = true;
                self.towards_min(right);
            }
            _ => {
                *visited_right_subtree = true;
                self.upwards()
            }
        };

        Some((&node.key, &node.value))
    }
}

#[derive(Clone, Copy)]
enum Dir {
    Left,
    Right,
}

impl Dir {
    #[inline]
    fn flip(self) -> Self {
        match self {
            Dir::Right => Dir::Left,
            Dir::Left => Dir::Right,
        }
    }
}

#[derive(Clone, Copy)]
enum Path {
    Empty,
    One(Dir),
    Two(Dir, Dir),
}

impl Path {
    #[inline]
    fn extend(&mut self, dir: Dir) {
        match *self {
            Path::Empty => *self = Path::One(dir),
            Path::One(dir1) => *self = Path::Two(dir, dir1),
            Path::Two(dir1, _) => *self = Path::Two(dir, dir1),
        }
    }
}

enum OrCreate<'a, K, V> {
    Lookup(&'a K),
    Create(K, V),
}

impl<'a, K, V> OrCreate<'a, K, V> {
    #[inline]
    fn key(&self) -> &K {
        match self {
            OrCreate::Lookup(k) => k,
            OrCreate::Create(k, _) => &k,
        }
    }

    #[inline]
    fn value(self) -> Option<V> {
        match self {
            OrCreate::Lookup(_) => None,
            OrCreate::Create(_, value) => Some(value),
        }
    }
}

impl<K: Ord, V> Splay<K, V> {
    pub fn new() -> Self {
        Splay {
            root: IDX_NONE,
            nodes: Vec::new(),
        }
    }

    fn node_depth(&self, idx: OptionIdx) -> u32 {
        match idx.to_option() {
            None => 0,
            Some(idx) => {
                1 + std::cmp::max(
                    self.node_depth(self.nodes[idx].left),
                    self.node_depth(self.nodes[idx].right),
                )
            }
        }
    }

    pub fn depth(&self) -> u32 {
        self.node_depth(self.root)
    }

    pub fn iter(&self) -> SplayIter<K, V> {
        SplayIter::new(self)
    }

    #[inline]
    fn child(&self, idx: Idx, dir: Dir) -> OptionIdx {
        match dir {
            Dir::Left => self.nodes[idx as usize].left,
            Dir::Right => self.nodes[idx as usize].right,
        }
    }

    #[inline]
    fn set_child(&mut self, idx: Idx, dir: Dir, to: OptionIdx) {
        match dir {
            Dir::Left => self.nodes[idx as usize].left = to,
            Dir::Right => self.nodes[idx as usize].right = to,
        };
    }

    pub fn get(&mut self, key: K) -> Option<&V> {
        self.visit(OrCreate::Lookup(&key));
        self.root.to_option().and_then(|root| {
            if self.nodes[root as usize].key == key {
                Some(&self.nodes[root as usize].value)
            } else {
                None
            }
        })
    }

    #[inline]
    fn new_node(&mut self, key: K, value: V) -> Idx {
        let node = Node {
            key,
            value,
            left: IDX_NONE,
            right: IDX_NONE,
        };
        self.nodes.push(node);
        (self.nodes.len() - 1) as Idx
    }

    #[inline]
    /// Swaps upper with lower.
    fn rotate(&mut self, upper: Idx, dir: Dir) {
        let lower = self.child(upper, dir).to_option().unwrap();

        self.set_child(upper, dir, self.child(lower, dir.flip()));
        self.set_child(lower, dir.flip(), OptionIdx(lower));

        self.nodes.swap(upper, lower);
    }

    #[inline]
    fn splay_step(&mut self, idx: Idx, path: &mut Path) {
        match *path {
            Path::Empty | Path::One(_) => {}
            Path::Two(dir1, dir2) => {
                let next_node = self.child(idx, dir1).to_option().unwrap();
                self.rotate(next_node, dir2);
                self.rotate(idx, dir1);
                *path = Path::Empty;
            }
        }
    }

    #[inline]
    fn splay_finish(&mut self, path: &Path) {
        match path {
            Path::Empty => {}
            Path::Two(..) => unreachable!(),
            Path::One(dir) => {
                let root = self.root.to_option().unwrap();
                self.rotate(root, *dir)
            }
        }
    }

    #[inline]
    fn visit_inner_helper(
        &mut self,
        node_idx: Idx,
        create: OrCreate<K, V>,
        dir: Dir,
        path: &mut Path,
    ) -> Option<V> {
        match self.child(node_idx, dir).to_option() {
            Some(idx) => {
                let value = self.visit_inner(idx, create, path);
                path.extend(dir);
                value
            }
            None => {
                if let OrCreate::Create(k, v) = create {
                    let node = self.new_node(k, v);
                    self.set_child(node_idx, dir, OptionIdx(node));
                    *path = Path::One(dir)
                }
                None
            }
        }
    }

    #[inline]
    fn visit_inner(&mut self, node_idx: Idx, create: OrCreate<K, V>, path: &mut Path) -> Option<V> {
        let key = create.key();

        let value = match key.cmp(&self.nodes[node_idx as usize].key) {
            Equal => {
                *path = Path::Empty;
                create.value()
            }
            Less => self.visit_inner_helper(node_idx, create, Dir::Left, path),
            Greater => self.visit_inner_helper(node_idx, create, Dir::Right, path),
        };

        self.splay_step(node_idx, path);
        value
    }

    fn visit(&mut self, create: OrCreate<K, V>) -> Option<V> {
        match self.root.to_option() {
            Some(root) => {
                let mut path = Path::Empty;
                let value = self.visit_inner(root, create, &mut path);
                self.splay_finish(&path);
                value
            }
            None => match create {
                OrCreate::Lookup(_) => None,
                OrCreate::Create(key, value) => {
                    let root = self.new_node(key, value);
                    self.root = OptionIdx(root);
                    None
                }
            },
        }
    }

    pub fn set(&mut self, key: K, value: V) {
        if let Some(value) = self.visit(OrCreate::Create(key, value)) {
            self.nodes[self.root.to_option().unwrap() as usize].value = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{Arbitrary, Gen};
    use rand::seq::SliceRandom;
    use std::collections::HashMap;

    use super::*;
    use quickcheck_macros::quickcheck;

    #[test]
    fn basic_test() {
        let mut tree: Splay<i32, i32> = Splay::new();
        tree.set(1, 1);
        tree.set(2, 2);
        assert_eq!(tree.get(1), Some(&1));
        assert_eq!(tree.get(2), Some(&2));
        assert_eq!(tree.get(3), None);
        tree.set(2, 1);
        assert_eq!(tree.get(2), Some(&1));
        assert_eq!(
            tree.iter()
                .map(|(x, y)| (*x, *y))
                .collect::<Vec<(i32, i32)>>(),
            vec![(1, 1), (2, 1)]
        );
    }

    #[test]
    fn depth_test() {
        let mut rng = rand::rng();
        let mut tree: Splay<i32, i32> = Splay::new();
        let mut keys: Vec<i32> = (1..100000).collect();
        keys.shuffle(&mut rng);
        for key in keys {
            tree.set(key, key);
        }

        let depth = tree.depth();
        println!("depth: {}", depth);
        assert!(depth < 50);
    }

    #[derive(Clone, Debug)]
    enum Op {
        Set(i32, i32),
        Get(i32),
        CompareSorted,
    }

    impl Arbitrary for Op {
        fn arbitrary(g: &mut Gen) -> Self {
            match *g.choose(&[0, 1, 2]).unwrap() {
                0 => Op::Set(i32::arbitrary(g), i32::arbitrary(g)),
                1 => Op::Get(i32::arbitrary(g)),
                2 => Op::CompareSorted,
                _ => unreachable!(),
            }
        }
    }

    #[quickcheck]
    fn test_quickcheck(ops: Vec<Op>) -> bool {
        let mut tree: Splay<i32, i32> = Splay::new();
        let mut map: HashMap<i32, i32> = HashMap::new();

        for op in ops.iter() {
            match *op {
                Op::Set(k, v) => {
                    tree.set(k, v);
                    map.insert(k, v);
                }
                Op::Get(k) => {
                    if tree.get(k) != map.get(&k) {
                        return false;
                    }
                }
                Op::CompareSorted => {
                    let tree_vec: Vec<i32> = tree.iter().map(|(k, _)| *k).collect();
                    let mut map_vec: Vec<i32> = map.iter().map(|(k, _)| *k).collect();
                    map_vec.sort();
                    assert_eq!(tree_vec, map_vec);
                }
            }
        }

        return true;
    }
}
