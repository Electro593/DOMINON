use std::{
    mem,
    ops::{Index, IndexMut},
};

#[derive(Copy, Clone, Debug)]
struct FreeNode {
    next: usize,
}

#[derive(Debug)]
pub struct ValueNode<T> {
    value: T,
    parent: usize,
    prev: usize,
    next: usize,
    child: usize,
    len: usize,
}

impl<T> ValueNode<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            parent: 0,
            prev: 0,
            next: 0,
            child: 0,
            len: 0,
        }
    }
}

#[derive(Debug)]
enum Node<T> {
    None,
    Free(FreeNode),
    Value(ValueNode<T>),
}

#[derive(Debug)]
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
    free: usize,
}

pub struct TreeNode<'a, T> {
    tree: &'a Tree<T>,
    index: usize,
    pub len: usize,
    pub value: &'a T,
}

pub struct TreeNodeMut<'a, T> {
    index: usize,
    pub value: &'a mut T,
}

impl<'a, T> TreeNode<'a, T> {
    #[must_use]
    fn new(tree: &'a Tree<T>, index: usize) -> Self {
        let node = &tree[index];
        Self {
            tree,
            index,
            len: node.len,
            value: &node.value,
        }
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<Self> {
        let child_index = self.tree.get_child(self.index, index).unwrap();
        self.tree.get(child_index).map(|child| Self {
            tree: self.tree,
            index: child_index,
            len: child.len,
            value: &child.value,
        })
    }
}

impl<'a, T> Index<usize> for TreeNode<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tree[self.tree.get_child(self.index, index).unwrap()].value
    }
}

impl<'a, T> TreeNodeMut<'a, T> {
    #[must_use]
    fn new(tree: &'a mut Tree<T>, index: usize) -> Self {
        Self {
            index,
            value: &mut tree[index].value,
        }
    }

    #[must_use]
    pub fn get(&self, tree: &'a mut Tree<T>, index: usize) -> Option<Self> {
        let child_index = tree.get_child(self.index, index).unwrap();
        tree.get_mut(child_index).map(|child| Self {
            index: child_index,
            value: &mut child.value,
        })
    }

    pub fn insert(&self, tree: &'a mut Tree<T>, index: usize, value: T) -> Self {
        let new_index = tree.insert(self.index, index, value).unwrap();
        Self {
            index: new_index,
            value: &mut tree[new_index].value,
        }
    }

    pub fn remove(&self, tree: &'a mut Tree<T>, index: usize) -> Option<T> {
        tree.remove(tree.get_child(self.index, index).unwrap())
    }
}

impl<T> Tree<T> {
    #[must_use]
    pub fn new(root: T) -> Self {
        Self {
            nodes: vec![Node::None, Node::Value(ValueNode::new(root))],
            free: 0,
        }
    }

    #[must_use]
    pub fn root(&self) -> TreeNode<'_, T> {
        TreeNode::new(self, 1)
    }

    #[must_use]
    pub fn root_mut(&mut self) -> TreeNodeMut<'_, T> {
        TreeNodeMut::new(self, 1)
    }

    #[must_use]
    fn get(&self, index: usize) -> Option<&ValueNode<T>> {
        self.nodes.get(index).and_then(|n| match n {
            Node::Value(value) => Some(value),
            _ => None,
        })
    }

    #[must_use]
    fn get_mut(&mut self, index: usize) -> Option<&mut ValueNode<T>> {
        self.nodes.get_mut(index).and_then(|n| match n {
            Node::Value(value) => Some(value),
            _ => None,
        })
    }

    #[must_use]
    fn alloc(&mut self, value: T) -> usize {
        let node = Node::Value(ValueNode::new(value));

        let index = self.free;
        match self.nodes[index] {
            Node::None => {
                self.nodes.push(node);
            }
            Node::Free(free) => {
                self.free = free.next;
                self.nodes[index] = node;
            }
            _ => panic!(),
        }

        index
    }

    #[must_use]
    fn get_child(&self, parent_index: usize, index: usize) -> Option<usize> {
        let mut child_index = self.get(parent_index)?.child;
        for _ in 0..index {
            if let Some(child) = self.get(child_index) {
                child_index = child.next;
            }
        }
        Some(child_index)
    }

    fn insert(&mut self, parent_index: usize, index: usize, value: T) -> Option<usize> {
        let new_index = self.alloc(value);
        self[new_index].parent = parent_index;

        let child_index = self.get(parent_index)?.child;
        match self.get(child_index) {
            Some(child) => {
                let mut last_index = child_index;
                let mut next_index = child.next;
                for _ in 0..index {
                    if let Some(child) = self.get(next_index) {
                        last_index = next_index;
                        next_index = child.next;
                    }
                }
                self[last_index].next = new_index;
                self[new_index].prev = last_index;
            }
            None => {
                self[parent_index].child = new_index;
            }
        }

        self[parent_index].len += 1;
        Some(new_index)
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        let node = self.get(index)?;
        let (parent_index, prev_index, next_index, child_index) =
            (node.parent, node.prev, node.next, node.child);

        let free = Node::Free(FreeNode { next: self.free });
        self.free = index;

        if self[parent_index].child == index {
            self[parent_index].child = next_index;
        }
        if let Some(prev) = self.get_mut(prev_index) {
            prev.next = next_index
        };
        if let Some(next) = self.get_mut(next_index) {
            next.prev = prev_index
        };

        let mut stack = vec![child_index];
        while let Some(i) = stack.pop() {
            if let Some(curr) = self.get(i) {
                stack.push(curr.next);
                stack.push(curr.child);

                self.nodes[i] = Node::Free(FreeNode { next: self.free });
                self.free = i;
            }
        }

        self[parent_index].len -= 1;

        match mem::replace(&mut self.nodes[index], free) {
            Node::Value(used) => Some(used.value),
            _ => panic!(),
        }
    }
}

impl<T> Index<usize> for Tree<T> {
    type Output = ValueNode<T>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<usize> for Tree<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
