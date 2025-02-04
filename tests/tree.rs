/****************************************************************************** 
 * Copyright 2025 ContinuousC                                                 * 
 *                                                                            * 
 * Licensed under the Apache License,  Version 2.0  (the "License");  you may * 
 * not use this file except in compliance with the License. You may  obtain a * 
 * copy of the License at http://www.apache.org/licenses/LICENSE-2.0          * 
 *                                                                            * 
 * Unless  required  by  applicable  law  or agreed  to in  writing, software * 
 * distributed under the License is distributed on an "AS IS"  BASIS, WITHOUT * 
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express  or implied.  See the * 
 * License for the  specific language  governing permissions  and limitations * 
 * under the License.                                                         * 
 ******************************************************************************/

use std::{collections::BTreeMap, fmt::Write};

use graph::{BTreeGraph, Ref};

struct Tree(BTreeGraph<String, Node>);

#[derive(Clone)]
struct NamedPtr(String, graph::Ref<Node>);
struct NamedRef<'a>(&'a str, &'a Node);
// struct NamedMut<'a>(&'a str, &'a mut Node);

struct Node {
    parent: Option<NamedPtr>,
    children: BTreeMap<String, Ref<Node>>,
}

impl Tree {
    fn new() -> Self {
        Self(BTreeGraph::new())
    }

    fn insert(&mut self, key: String, value: Node) -> NamedPtr {
        NamedPtr(key.clone(), self.0.insert(key, value))
    }

    fn insert_child(&mut self, parent: NamedPtr, key: String, value: Node) -> NamedPtr {
        let child = self.0.insert(key.clone(), value);

        self.0
            .borrow_mut(&parent.1)
            .children
            .insert(key.clone(), child.clone());
        self.0.borrow_mut(&child).parent = Some(parent);

        NamedPtr(key, child)
    }

    fn show(&self, root: NamedPtr) -> String {
        let mut out = String::new();
        root.get(self).show(&mut out, self, 0).unwrap();
        out
    }
}

impl Node {
    fn new() -> Self {
        Self {
            parent: None,
            children: BTreeMap::new(),
        }
    }
}

impl NamedPtr {
    fn get<'a>(&'a self, tree: &'a Tree) -> NamedRef<'a> {
        NamedRef(&self.0, tree.0.borrow(&self.1))
    }
    // fn get_mut<'a>(&'a self, tree: &'a mut Tree) -> NamedMut<'a> {
    //     NamedMut(&self.0, tree.0.get_mut(&self.1))
    // }
}

impl<'a> NamedRef<'a> {
    fn key(&self) -> &str {
        self.0
    }

    fn value(&self) -> &Node {
        self.1
    }

    fn show(&self, output: &mut String, tree: &Tree, indent: usize) -> std::fmt::Result {
        (0..indent).try_for_each(|_| write!(output, "  "))?;
        write!(output, "{}", self.key())?;
        match &self.value().parent {
            Some(parent) => writeln!(output, " (parent: {})", parent.get(tree).key())?,
            None => writeln!(output)?,
        }
        self.1.children.iter().try_for_each(|(name, child)| {
            NamedRef(name, tree.0.borrow(child)).show(output, tree, indent + 1)
        })
    }
}

// impl<'a> NamedMut<'a> {
//     // fn key(&self) -> &str {
//     //     self.0.key()
//     // }

//     // fn value(&self) -> &Node {
//     //     self.0.value()
//     // }

//     fn value_mut(&mut self) -> &mut Node {
//         self.1
//     }
// }

#[test]
fn double_linked_tree() {
    let mut tree = Tree::new();
    let root = tree.insert("root".to_string(), Node::new());
    let child1 = tree.insert_child(root.clone(), "child 1".to_string(), Node::new());
    let _child2 = tree.insert_child(root.clone(), "child 2".to_string(), Node::new());
    let child11 = tree.insert_child(child1.clone(), "child 1.1".to_string(), Node::new());
    let _child12 = tree.insert_child(child1, "child 1.2".to_string(), Node::new());
    let _child111 = tree.insert_child(child11, "child 1.1.1".to_string(), Node::new());

    assert_eq!(
        &tree.show(root),
        r#"root
  child 1 (parent: root)
    child 1.1 (parent: child 1)
      child 1.1.1 (parent: child 1.1)
    child 1.2 (parent: child 1)
  child 2 (parent: root)
"#
    )
}
