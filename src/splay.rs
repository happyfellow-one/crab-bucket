use std::cmp::Ordering;

struct Node<K, V> {
    key: K,
    value: V,
    left: Option<usize>,
    right: Option<usize>,
}

struct Splay<K, V> {
    root: Option<usize>,
    nodes: Vec<Node<K, V>>,
}

#[derive(Clone, Copy)]
enum Dir {
    Left,
    Right,
}

impl Dir {
    fn flip(self) -> Self {
        match self {
            Dir::Right => Dir::Left,
            Dir::Left => Dir::Right,
        }
    }
}

#[derive(Clone, Copy)]
enum Path {
    Here(usize),
    One(usize, Dir, usize),
    Two(usize, Dir, usize, Dir, usize),
}

impl Path {
    fn here(&self) -> usize {
        use Path::*;
        match *self {
            Here(idx) | One(idx, ..) | Two(idx, ..) => idx,
        }
    }

    fn extend(self, node_idx: usize, dir: Dir) -> Self {
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
    fn key(&self) -> &K {
        match self {
            OrCreate::Lookup(k) => k,
            OrCreate::Create(k, _) => &k,
        }
    }

    fn value(self) -> Option<V> {
        match self {
            OrCreate::Lookup(_) => None,
            OrCreate::Create(_, value) => Some(value),
        }
    }
}

impl<K: Ord, V> Splay<K, V> {
    fn new() -> Self {
        Splay {
            root: None,
            nodes: Vec::new(),
        }
    }

    fn child(&self, idx: usize, dir: Dir) -> Option<usize> {
        match dir {
            Dir::Left => self.nodes[idx].left,
            Dir::Right => self.nodes[idx].right,
        }
    }

    fn set_child(&mut self, idx: usize, dir: Dir, to: Option<usize>) {
        match dir {
            Dir::Left => self.nodes[idx].left = to,
            Dir::Right => self.nodes[idx].right = to,
        }
    }

    fn get(&mut self, key: K) -> Option<&V> {
        self.visit(OrCreate::Lookup(&key));
        self.root.and_then(|root| {
            if self.nodes[root].key == key {
                Some(&self.nodes[root].value)
            } else {
                None
            }
        })
    }

    fn new_node(&mut self, key: K, value: V) -> usize {
        let node = Node {
            key,
            value,
            left: None,
            right: None,
        };
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    fn rotate(&mut self, upper: usize, dir: Dir, lower: usize) -> usize {
        assert_eq!(self.child(upper, dir), Some(lower));

        self.set_child(upper, dir, self.child(lower, dir.flip()));
        self.set_child(lower, dir.flip(), Some(upper));

        lower
    }

    fn splay_step(&mut self, path: Path) -> Path {
        use Path::*;

        match path {
            Two(idx1, dir1, idx2, dir2, idx3) => {
                let middle = self.rotate(idx2, dir2, idx3);
                self.set_child(idx1, dir1, Some(middle));
                let new_idx = self.rotate(idx1, dir1, middle);
                Here(new_idx)
            }
            Here(_) | One(..) => path,
        }
    }

    fn splay_finish(&mut self, path: Path) {
        let root = match path {
            Path::Here(root) => root,
            Path::One(root, dir, x) => self.rotate(root, dir, x),
            Path::Two(..) => unreachable!(),
        };
        self.root = Some(root);
    }

    fn visit_inner(
        &mut self,
        node_idx: Option<usize>,
        create: OrCreate<K, V>,
    ) -> (Option<Path>, Option<V>) {
        let key = create.key();

        let (path, value) = match node_idx {
            None => {
                if let OrCreate::Create(key, value) = create {
                    let new_node_idx = self.new_node(key, value);
                    (Some(Path::Here(new_node_idx)), None)
                } else {
                    (None, None)
                }
            }
            Some(node_idx) => {
                let cmp_result = key.cmp(&self.nodes[node_idx].key);
                let mut recurse = |dir: Dir, create: OrCreate<K, V>| -> (Option<Path>, Option<V>) {
                    let (path, value) = self.visit_inner(self.child(node_idx, dir), create);
                    (
                        path.map(|path| {
                            self.set_child(node_idx, dir, Some(path.here()));
                            path.extend(node_idx, dir)
                        }),
                        value,
                    )
                };

                match cmp_result {
                    Ordering::Equal => (Some(Path::Here(node_idx)), create.value()),
                    Ordering::Less => recurse(Dir::Left, create),
                    Ordering::Greater => recurse(Dir::Right, create),
                }
            }
        };

        let path = path.map(|path| self.splay_step(path));
        (path, value)
    }

    fn visit(&mut self, create: OrCreate<K, V>) -> Option<V> {
        let (path, value) = self.visit_inner(self.root, create);
        path.inspect(|path| self.splay_finish(*path));
        value
    }

    fn set(&mut self, key: K, value: V) {
        if let Some(value) = self.visit(OrCreate::Create(key, value)) {
            self.nodes[self.root.unwrap()].value = value;
        }
    }
}

#[cfg(test)]
mod tests {
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

    #[quickcheck]
    fn insert_and_get(elems: HashMap<i32, i32>) -> bool {
        let mut tree: Splay<i32, i32> = Splay::new();
        for (k, v) in elems.iter() {
            tree.set(*k, *v);
        }

        for (k, v) in elems.iter() {
            if tree.get(*k) != Some(v) {
                return false;
            }
        }
        return true;
    }
}
