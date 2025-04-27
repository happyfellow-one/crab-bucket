use std::cmp::Ordering;

type Idx = usize;

#[derive(Clone, Copy, Debug, PartialEq)]
struct OptionIdx(Idx);
const idx_none: OptionIdx = OptionIdx(Idx::MAX);

impl OptionIdx {
    #[inline]
    fn to_option(self) -> Option<Idx> {
        if self == idx_none {
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
/// A path from the current node to the node being splayed.
#[derive(Clone, Copy)]
enum Path {
    Here(Idx),
    One(Idx, Dir, Idx),
    Two(Idx, Dir, Idx, Dir, Idx),
}

impl Path {
    #[inline]
    fn here(&self) -> Idx {
        use Path::*;
        match *self {
            Here(idx) | One(idx, ..) | Two(idx, ..) => idx,
        }
    }

    #[inline]
    fn extend(self, node_idx: Idx, dir: Dir) -> Self {
        use Path::*;

        match self {
            Here(idx) => One(node_idx, dir, idx),
            One(idx1, dir1, idx2) => Two(node_idx, dir, idx1, dir1, idx2),
            Two(_, _, idx1, dir1, idx2) => Two(node_idx, dir, idx1, dir1, idx2),
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
            root: idx_none,
            nodes: Vec::new(),
        }
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
            Dir::Left => std::mem::replace(&mut self.nodes[idx as usize].left, to),
            Dir::Right => std::mem::replace(&mut self.nodes[idx as usize].right, to),
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
            left: idx_none,
            right: idx_none,
        };
        self.nodes.push(node);
        (self.nodes.len() - 1) as Idx
    }

    #[inline]
    /// Swaps upper with lower.
    fn rotate(&mut self, upper: Idx, dir: Dir) -> Idx {
        let lower = self.child(upper, dir).to_option().unwrap();

        self.set_child(upper, dir, self.child(lower, dir.flip()));
        self.set_child(lower, dir.flip(), OptionIdx(upper));

        lower
    }

    #[inline]
    fn splay_step(&mut self, path: Path) -> Path {
        use Path::*;

        match path {
            Two(idx1, dir1, idx2, dir2, idx3) => {
                let middle = self.rotate(idx2, dir2);
                self.set_child(idx1, dir1, OptionIdx(middle));
                let new_idx = self.rotate(idx1, dir1);
                Here(new_idx)
            }
            Here(_) | One(..) => path,
        }
    }

    #[inline]
    fn splay_finish(&mut self, path: Path) {
        let root = match path {
            Path::Here(root) => root,
            Path::One(root, dir, x) => self.rotate(root, dir),
            Path::Two(..) => unreachable!(),
        };
        self.root = OptionIdx(root);
    }

    #[inline]
    fn visit_inner_helper(
        &mut self,
        node_idx: Idx,
        create: OrCreate<K, V>,
        dir: Dir,
    ) -> (Option<Path>, Option<V>) {
        let (path, value) = self.visit_inner(self.child(node_idx, dir), create);
        (
            path.map(|path| {
                self.set_child(node_idx, dir, OptionIdx(path.here()));
                path.extend(node_idx, dir)
            }),
            value,
        )
    }

    #[inline]
    fn visit_inner(
        &mut self,
        node_idx: OptionIdx,
        create: OrCreate<K, V>,
    ) -> (Option<Path>, Option<V>) {
        let key = create.key();

        let (path, value) = match node_idx.to_option() {
            None => {
                if let OrCreate::Create(key, value) = create {
                    let new_node_idx = self.new_node(key, value);
                    (Some(Path::Here(new_node_idx)), None)
                } else {
                    (None, None)
                }
            }
            Some(node_idx) => match key.cmp(&self.nodes[node_idx as usize].key) {
                Ordering::Equal => (Some(Path::Here(node_idx)), create.value()),
                Ordering::Less => self.visit_inner_helper(node_idx, create, Dir::Left),
                Ordering::Greater => self.visit_inner_helper(node_idx, create, Dir::Right),
            },
        };

        let path = path.map(|path| self.splay_step(path));
        (path, value)
    }

    fn visit(&mut self, create: OrCreate<K, V>) -> Option<V> {
        let (path, value) = self.visit_inner(self.root, create);
        path.inspect(|path| self.splay_finish(*path));
        value
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
    }

    #[derive(Clone, Debug)]
    enum Op {
        Set(i32, i32),
        Get(i32),
    }

    impl Arbitrary for Op {
        fn arbitrary(g: &mut Gen) -> Self {
            match *g.choose(&[0, 1]).unwrap() {
                0 => Op::Set(i32::arbitrary(g), i32::arbitrary(g)),
                1 => Op::Get(i32::arbitrary(g)),
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
            }
        }

        return true;
    }
}
