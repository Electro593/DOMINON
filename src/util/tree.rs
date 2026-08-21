// use std::{
//     mem,
//     ops::{Index, IndexMut},
// };
//
// #[derive(Copy, Clone, Debug)]
// struct FreeNode {
//     next: usize,
// }
//
// #[derive(Debug)]
// pub struct TreeValueNode<T> {
//     pub value: T,
//     pub parent: usize,
//     pub prev: usize,
//     pub next: usize,
//     pub child: usize,
//     pub len: usize,
// }
//
// impl<T> TreeValueNode<T> {
//     fn new(value: T) -> Self {
//         Self {
//             value,
//             parent: 0,
//             prev: 0,
//             next: 0,
//             child: 0,
//             len: 0,
//         }
//     }
// }
//
// #[derive(Debug)]
// enum Node<T> {
//     None,
//     Free(FreeNode),
//     Value(TreeValueNode<T>),
// }
//
// #[derive(Debug)]
// pub struct Tree<T> {
//     nodes: Vec<Node<T>>,
//     free: usize,
// }
//
// #[derive(Debug)]
// pub struct TreeNode<'a, T> {
//     tree: &'a Tree<T>,
//     pub index: usize,
//     pub len: usize,
//     pub value: &'a T,
// }
//
// #[derive(Debug)]
// pub struct TreeNodeMut<'a, T> {
//     index: usize,
//     pub value: &'a mut T,
// }
//
// pub struct PostOrderTreeIter<'a, T> {
//     stack: Vec<(TreeNode<'a, T>, bool)>,
// }
//
// impl<'a, T> TreeNode<'a, T> {
//     #[must_use]
//     pub fn try_from(tree: &'a Tree<T>, index: usize) -> Option<Self> {
//         tree.get(index).map(|node| Self {
//             tree,
//             index,
//             len: node.len,
//             value: &node.value,
//         })
//     }
//
//     #[must_use]
//     pub fn parent(&self) -> Option<Self> {
//         Self::try_from(self.tree, self.tree[self.index].parent)
//     }
//
//     #[must_use]
//     pub fn prev(&self) -> Option<Self> {
//         Self::try_from(self.tree, self.tree[self.index].prev)
//     }
//
//     #[must_use]
//     pub fn next(&self) -> Option<Self> {
//         Self::try_from(self.tree, self.tree[self.index].next)
//     }
//
//     #[must_use]
//     pub fn child(&self, index: usize) -> Option<Self> {
//         self.tree
//             .get_child(self.index, index)
//             .and_then(|child| Self::try_from(self.tree, child))
//     }
//
//     #[must_use]
//     pub fn into_mut(self, tree: &'a mut Tree<T>) -> TreeNodeMut<'a, T> {
//         TreeNodeMut::try_from(tree, self.index).unwrap()
//     }
//
//     #[must_use]
//     pub fn iter_post_order(&self) -> PostOrderTreeIter<'a, T> {
//         PostOrderTreeIter {
//             stack: vec![(*self, false)],
//         }
//     }
// }
//
// impl<'a, T> Clone for TreeNode<'a, T> {
//     fn clone(&self) -> Self {
//         Self {
//             tree: self.tree,
//             index: self.index,
//             len: self.len,
//             value: self.value,
//         }
//     }
// }
//
// impl<'a, T> Copy for TreeNode<'a, T> {}
//
// impl<'a, T> Index<usize> for TreeNode<'a, T> {
//     type Output = T;
//
//     fn index(&self, index: usize) -> &Self::Output {
//         &self.tree[self.tree.get_child(self.index, index).unwrap()].value
//     }
// }
//
// impl<'a, T> TreeNodeMut<'a, T> {
//     #[must_use]
//     pub fn try_from(tree: &'a mut Tree<T>, index: usize) -> Option<Self> {
//         tree.get_mut(index).map(|node| Self {
//             index,
//             value: &mut node.value,
//         })
//     }
//
//     #[must_use]
//     pub fn parent(&self, tree: &'a mut Tree<T>) -> Option<Self> {
//         Self::try_from(tree, tree[self.index].parent)
//     }
//
//     #[must_use]
//     pub fn prev(&self, tree: &'a mut Tree<T>) -> Option<Self> {
//         Self::try_from(tree, tree[self.index].prev)
//     }
//
//     #[must_use]
//     pub fn next(&self, tree: &'a mut Tree<T>) -> Option<Self> {
//         Self::try_from(tree, tree[self.index].next)
//     }
//
//     #[must_use]
//     pub fn child(&self, tree: &'a mut Tree<T>, index: usize) -> Option<Self> {
//         tree.get_child(self.index, index)
//             .and_then(|child| Self::try_from(tree, child))
//     }
//
//     #[must_use]
//     pub fn into_immut(self, tree: &'a Tree<T>) -> TreeNode<'a, T> {
//         TreeNode::try_from(tree, self.index).unwrap()
//     }
//
//     pub fn insert(&self, tree: &'a mut Tree<T>, index: usize, value: T) -> Option<Self> {
//         tree.insert(self.index, index, value)
//             .and_then(|new_index| Self::try_from(tree, new_index))
//     }
//
//     pub fn remove(&self, tree: &'a mut Tree<T>, index: usize) -> Option<T> {
//         tree.get_child(self.index, index)
//             .and_then(|child| tree.remove(child))
//     }
// }
//
// impl<'a, T> Iterator for PostOrderTreeIter<'a, T> {
//     type Item = &'a T;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some((node, visited)) = self.stack.pop() {
//             if visited {
//                 return Some(node.value);
//             }
//
//             self.stack.push((node, true));
//             node.next().map(|n| self.stack.push((n, false)));
//             node.child(0).map(|n| self.stack.push((n, false)));
//         }
//         None
//     }
// }
//
// impl<T> Tree<T> {
//     #[must_use]
//     pub fn new(root: T) -> Self {
//         Self {
//             nodes: vec![Node::None, Node::Value(TreeValueNode::new(root))],
//             free: 0,
//         }
//     }
//
//     #[must_use]
//     pub fn root(&self) -> TreeNode<'_, T> {
//         TreeNode::try_from(self, 1).unwrap()
//     }
//
//     #[must_use]
//     pub fn root_mut(&mut self) -> TreeNodeMut<'_, T> {
//         TreeNodeMut::try_from(self, 1).unwrap()
//     }
//
//     #[must_use]
//     fn get(&self, index: usize) -> Option<&TreeValueNode<T>> {
//         self.nodes.get(index).and_then(|n| match n {
//             Node::Value(value) => Some(value),
//             _ => None,
//         })
//     }
//
//     #[must_use]
//     fn get_mut(&mut self, index: usize) -> Option<&mut TreeValueNode<T>> {
//         self.nodes.get_mut(index).and_then(|n| match n {
//             Node::Value(value) => Some(value),
//             _ => None,
//         })
//     }
//
//     #[must_use]
//     fn alloc(&mut self, value: T) -> usize {
//         let node = Node::Value(TreeValueNode::new(value));
//
//         let index = self.free;
//         match self.nodes[index] {
//             Node::None => {
//                 self.nodes.push(node);
//             }
//             Node::Free(free) => {
//                 self.free = free.next;
//                 self.nodes[index] = node;
//             }
//             _ => panic!(),
//         }
//
//         index
//     }
//
//     #[must_use]
//     fn get_child(&self, parent_index: usize, index: usize) -> Option<usize> {
//         let mut child_index = self.get(parent_index)?.child;
//         for _ in 0..index {
//             child_index = self.get(child_index)?.next;
//         }
//         self.get(child_index).map(|_| child_index)
//     }
//
//     fn insert(&mut self, parent_index: usize, index: usize, value: T) -> Option<usize> {
//         let child_index = self.get(parent_index)?.child;
//
//         let (prev_index, next_index) = if index == 0 {
//             (0, child_index)
//         } else {
//             let mut prev_index = child_index;
//             for _ in 1..index {
//                 prev_index = self.get(prev_index)?.next;
//             }
//             (prev_index, self.get(prev_index)?.next)
//         };
//
//         let new_index = self.alloc(value);
//         self[new_index].parent = parent_index;
//         self[parent_index].len += 1;
//         if index == 0 {
//             self[parent_index].child = new_index;
//         }
//
//         self[new_index].prev = prev_index;
//         self.get_mut(prev_index).map(|prev| prev.next = new_index);
//
//         self[new_index].next = next_index;
//         self.get_mut(next_index).map(|next| next.prev = new_index);
//
//         Some(new_index)
//     }
//
//     fn remove(&mut self, index: usize) -> Option<T> {
//         let node = self.get(index)?;
//         let (parent_index, prev_index, next_index, child_index) =
//             (node.parent, node.prev, node.next, node.child);
//
//         let free = Node::Free(FreeNode { next: self.free });
//         self.free = index;
//
//         if self[parent_index].child == index {
//             self[parent_index].child = next_index;
//         }
//         if let Some(prev) = self.get_mut(prev_index) {
//             prev.next = next_index
//         };
//         if let Some(next) = self.get_mut(next_index) {
//             next.prev = prev_index
//         };
//
//         let mut stack = vec![child_index];
//         while let Some(i) = stack.pop() {
//             if let Some(curr) = self.get(i) {
//                 stack.push(curr.next);
//                 stack.push(curr.child);
//
//                 self.nodes[i] = Node::Free(FreeNode { next: self.free });
//                 self.free = i;
//             }
//         }
//
//         self[parent_index].len -= 1;
//
//         match mem::replace(&mut self.nodes[index], free) {
//             Node::Value(used) => Some(used.value),
//             _ => panic!(),
//         }
//     }
// }
//
// impl<T> Index<usize> for Tree<T> {
//     type Output = TreeValueNode<T>;
//
//     fn index(&self, index: usize) -> &Self::Output {
//         self.get(index).unwrap()
//     }
// }
//
// impl<T> IndexMut<usize> for Tree<T> {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         self.get_mut(index).unwrap()
//     }
// }
