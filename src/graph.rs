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

use std::ptr::NonNull;
#[cfg(feature = "serde")]
use std::{fmt::Formatter, marker::PhantomData};

#[cfg(feature = "serde")]
use serde::{
    de::{Deserializer, SeqAccess, Visitor},
    Deserialize,
};
#[cfg(feature = "tsify")]
use tsify::Tsify;
use typed_arena::Arena;

use crate::{Gen, Ref};

#[cfg_attr(feature = "tsify", derive(Tsify))]
#[cfg_attr(feature = "tsify", tsify(from_wasm_abi, into_wasm_abi, type = "[T]"))]
pub struct Graph<T> {
    nodes: Arena<Option<T>>,
    gen: Gen,
}

impl<T> Graph<T> {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self {
            nodes: Arena::new(),
            gen: Gen::new(),
        }
    }

    /// Create an empty graph with capacity for ''n'' nodes.
    pub fn with_capacity(n: usize) -> Self {
        Self {
            nodes: Arena::with_capacity(n),
            gen: Gen::new(),
        }
    }

    /// Insert a node into the graph. The returned reference can be used
    /// to access this node.
    pub fn insert(&mut self, value: T) -> Ref<T> {
        unsafe {
            let node = self.nodes.alloc(Some(value));
            Ref::new(NonNull::new_unchecked(node), self.gen)
        }
    }

    /// Reserve an empty slot in the graph. This can be used when
    /// initializing the graph or to create cycles. Trying to access
    /// the node before it's value is set, will cause a panic.
    pub fn promise(&mut self) -> Ref<T> {
        unsafe {
            let node = self.nodes.alloc(None);
            Ref::new(NonNull::new_unchecked(node), self.gen)
        }
    }

    /// Create a node that has previously been promised or
    /// removed. Panics if the node already exists.
    pub fn create(&mut self, node: &Ref<T>, value: T) {
        #[cfg(any(not(feature = "unsafe"), debug_assertions))]
        assert!(self.gen == node.gen);
        let r = unsafe { node.try_replace_unchecked(value) };
        #[cfg(any(not(feature = "unsafe"), debug_assertions))]
        assert!(r.is_none());
    }

    /// Remove the value from the graph. Panics if you try to remove
    /// the node from a different graph or if the node was previously
    /// removed.
    pub fn remove(&mut self, node: Ref<T>) -> T {
        #[cfg(any(not(feature = "unsafe"), debug_assertions))]
        assert!(self.gen == node.gen);
        unsafe { node.try_remove_unchecked().unwrap() }
    }

    /// Borrow the value from the graph. Panics if you try to borrow
    /// the node from a different graph or if the node was previously
    /// removed.
    pub fn borrow<R>(&self, node: &R) -> &T
    where
        R: AsRef<Ref<T>>,
    {
        #[cfg(any(not(feature = "unsafe"), debug_assertions))]
        assert!(self.gen == node.as_ref().gen);
        unsafe { node.as_ref().try_get_unchecked().unwrap() }
    }

    /// Mutably borrow the value from the graph. Panics if you try to
    /// borrow the node from a different graph or if the node was
    /// previously removed.
    pub fn borrow_mut<R>(&mut self, node: &R) -> &mut T
    where
        R: AsRef<Ref<T>>,
    {
        #[cfg(any(not(feature = "unsafe"), debug_assertions))]
        assert!(self.gen == node.as_ref().gen);
        unsafe { node.as_ref().try_get_unchecked_mut().unwrap() }
    }

    /// Get mutable references to multiple nodes in the graph. This
    /// may be necessary to create cycles.
    pub fn borrow_many_mut<const N: usize, R>(&mut self, nodes: [R; N]) -> [&mut T; N]
    where
        R: AsRef<Ref<T>>,
    {
        #[cfg(any(not(feature = "unsafe"), debug_assertions))]
        assert!(nodes
            .iter()
            .enumerate()
            .all(|(i, node)| self.gen == node.as_ref().gen
                && nodes[..i]
                    .iter()
                    .all(|other| other.as_ref() != node.as_ref())));
        unsafe { nodes.map(|node| node.as_ref().try_get_unchecked_mut().unwrap()) }
    }

    /* Disabled because this needs invalid_reference_casting due to
     * Arena's lack of immutable iteration method. */
    // pub fn iter(&self) -> impl Iterator<Item = &T> {
    //     unsafe {
    //         // Arena provides an ``iter_mut``, but not an ``iter``
    //         // method, probably because it is !Sync and allows
    //         // allocation (mutation) through &self. Since our public
    //         // api only allows modification through methods taking
    //         // &mut self (enforcing exclusive access), this is
    //         // presumably fine.
    //         #[allow(invalid_reference_casting)]
    //         let self_mut = &mut *(self as *const Self as *mut Self);
    //         self_mut.nodes.iter_mut().flat_map(|node| &*node)
    //     }
    // }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.nodes.iter_mut().flatten()
    }
}

impl<T> Default for Graph<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IntoIterator for Graph<T> {
    type Item = T;
    type IntoIter = std::iter::Flatten<std::vec::IntoIter<Option<T>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_vec().into_iter().flatten()
    }
}

/* Safety: Graph is Send + Sync because the public interface only
 * allows modifying state when given a mutable reference. */

unsafe impl<T> Send for Graph<T> {}
unsafe impl<T> Sync for Graph<T> {}

/* Safety: Ref is Send + Sync because it cannot be modified using the
 * public interface. */

unsafe impl<T> Send for Ref<T> {}
unsafe impl<T> Sync for Ref<T> {}

/* This needs Graph::iter which is unsound due to Arena's lack of
 * immutable iteration method.  */
// #[cfg(feature = "serde")]
// impl<T> Serialize for Graph<T>
// where
//     T: Serialize,
// {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut s = serializer.serialize_seq(Some(self.iter().count()))?;
//         self.iter().try_for_each(|node| s.serialize_element(node))?;
//         s.end()
//     }
// }

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for Graph<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GraphVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for GraphVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Graph<T>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a sequence of nodes")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut graph = match seq.size_hint() {
                    Some(size) => Graph::with_capacity(size),
                    None => Graph::new(),
                };

                while let Some(node) = seq.next_element()? {
                    graph.insert(node);
                }

                Ok(graph)
            }
        }

        deserializer.deserialize_seq(GraphVisitor(PhantomData))
    }
}

#[cfg(test)]
mod test {

    use crate::{Graph, Ref};

    #[test]
    fn cycle() {
        #[derive(Debug)]
        struct Node {
            prev: Ref<Node>,
            next: Ref<Node>,
        }

        let mut graph = Graph::new();
        let a = graph.promise();
        let b = graph.promise();
        let c = graph.promise();

        graph.create(
            &a,
            Node {
                prev: c.clone(),
                next: b.clone(),
            },
        );
        graph.create(
            &b,
            Node {
                prev: a.clone(),
                next: c.clone(),
            },
        );
        graph.create(
            &c,
            Node {
                prev: b.clone(),
                next: a.clone(),
            },
        );

        assert_eq!(graph.borrow(&a).next, b);
        assert_eq!(graph.borrow(&b).next, c);
        assert_eq!(graph.borrow(&c).next, a);

        assert_eq!(graph.borrow(&c).prev, b);
        assert_eq!(graph.borrow(&b).prev, a);
        assert_eq!(graph.borrow(&a).prev, c);
    }
}
