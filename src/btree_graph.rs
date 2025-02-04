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

use std::{
    borrow::Borrow,
    collections::{btree_map, BTreeMap},
};
#[cfg(feature = "serde")]
use std::{fmt::Formatter, marker::PhantomData};

#[cfg(feature = "serde")]
use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
#[cfg(feature = "tsify")]
use tsify::Tsify;

use crate::reference::Ref;
use crate::{graph::Graph, RefBy};

/// A graph structure that allows pointer-based references between
/// nodes.
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[cfg_attr(
    feature = "tsify",
    tsify(from_wasm_abi, into_wasm_abi, type = "{ [key: K]: V }")
)]
pub struct BTreeGraph<K, V> {
    graph: Graph<V>,
    index: BTreeMap<K, Ref<V>>,
}

pub struct Entry<'a, K, V> {
    graph: &'a mut Graph<V>,
    entry: btree_map::Entry<'a, K, Ref<V>>,
}

impl<K, V> BTreeGraph<K, V> {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            index: BTreeMap::new(),
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        Self {
            graph: Graph::with_capacity(n),
            index: BTreeMap::new(), // ::with_capacity(n)
        }
    }

    pub fn index(&self) -> &BTreeMap<K, Ref<V>> {
        &self.index
    }

    /// Insert a node into the graph. The returned NodePtr can be used
    /// to reference this node.
    pub fn insert(&mut self, key: K, value: V) -> Ref<V>
    where
        K: Ord,
    {
        let node = self.graph.insert(value);
        if let Some(old_node) = self.index.insert(key, node.clone()) {
            unsafe {
                old_node.try_remove_unchecked().unwrap();
            }
        }
        node
    }

    pub fn promise(&mut self, key: K) -> Ref<V>
    where
        K: Ord,
    {
        let node = self.graph.promise();
        if let Some(old_node) = self.index.insert(key, node.clone()) {
            unsafe {
                old_node.try_remove_unchecked().unwrap();
            }
        }
        node
    }

    /// Remove a node from the graph. You are responsible to make sure
    /// no pointers to the node will be dereferenced from this point
    /// on.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        unsafe {
            let node = self.index.remove(key)?;
            Some(node.try_remove_unchecked().unwrap())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn get_ref<Q>(&self, key: &Q) -> Option<&Ref<V>>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.index.get(key)
    }

    pub fn get_ref_by<Q>(&self, key: &Q) -> Option<RefBy<K, V>>
    where
        K: Borrow<Q> + Ord + Clone,
        Q: Ord + ?Sized,
    {
        let (key, value) = self.index.get_key_value(key)?;
        Some(RefBy::new(key.clone(), value.clone()))
    }

    pub fn get_entry<Q>(&self, key: &Q) -> Option<(&K, &Ref<V>)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.index.get_key_value(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        unsafe { Some(self.get_ref(key)?.try_get_unchecked().unwrap()) }
    }

    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let (key, value) = self.get_entry(key)?;
        unsafe { Some((key, value.try_get_unchecked().unwrap())) }
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        unsafe { Some(self.get_ref(key)?.try_get_unchecked_mut().unwrap()) }
    }

    pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let (key, value) = self.get_entry(key)?;
        unsafe { Some((key, value.try_get_unchecked_mut().unwrap())) }
    }

    pub fn create(&mut self, node: &Ref<V>, value: V) {
        self.graph.create(node, value)
    }

    /// Borrow the value from the graph. Panics if you try to borrow
    /// the node from a different graph or if the node was previously
    /// removed.
    pub fn borrow<R>(&self, node: &R) -> &V
    where
        R: AsRef<Ref<V>>,
    {
        self.graph.borrow(node)
    }

    /// Mutably borrow the value from the graph. Panics if you try to
    /// borrow the node from a different graph or if the node was
    /// previously removed.
    pub fn borrow_mut<R>(&mut self, node: &R) -> &mut V
    where
        R: AsRef<Ref<V>>,
    {
        self.graph.borrow_mut(node)
    }

    /// Get mutable references to multiple nodes in the graph. This
    /// may be necessary to create cycles.
    pub fn borrow_many_mut<const N: usize, R>(&mut self, nodes: [R; N]) -> [&mut V; N]
    where
        K: PartialEq,
        R: AsRef<Ref<V>>,
    {
        self.graph.borrow_many_mut(nodes)
    }

    pub fn iter_ref(&self) -> impl Iterator<Item = (&K, &Ref<V>)> {
        self.index.iter()
    }

    pub fn iter_ref_by(&self) -> impl Iterator<Item = RefBy<K, V>> + '_
    where
        K: Clone,
    {
        self.iter_ref()
            .map(|(k, v)| RefBy::new(k.clone(), v.clone()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        unsafe {
            self.iter_ref()
                .map(|(key, value)| (key, value.try_get_unchecked().unwrap()))
        }
    }

    pub fn iter_mut(&self) -> impl Iterator<Item = (&K, &mut V)> {
        unsafe {
            self.iter_ref()
                .map(|(key, value)| (key, value.try_get_unchecked_mut().unwrap()))
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.index.keys()
    }

    pub fn values_ref(&self) -> impl Iterator<Item = &Ref<V>> {
        self.index.values()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        //self.graph.iter_mut()
        unsafe {
            self.values_ref()
                .map(|value| value.try_get_unchecked().unwrap())
        }
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        //self.graph.iter_mut()
        unsafe {
            self.values_ref()
                .map(|value| value.try_get_unchecked_mut().unwrap())
        }
    }

    pub fn entry(&mut self, key: K) -> Entry<K, V>
    where
        K: Ord,
    {
        Entry {
            graph: &mut self.graph,
            entry: self.index.entry(key),
        }
    }
}

impl<K, V> Default for BTreeGraph<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> AsRef<Graph<V>> for BTreeGraph<K, V> {
    fn as_ref(&self) -> &Graph<V> {
        &self.graph
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for BTreeGraph<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let ((_, Some(size)) | (size, None)) = iter.size_hint();

        let mut graph = Graph::with_capacity(size);
        let mut index = BTreeMap::new(); // ::with_capacity(size)

        iter.for_each(|(key, value)| {
            index.insert(key, graph.insert(value));
        });

        Self { graph, index }
    }
}

#[cfg(feature = "serde")]
impl<K, V> Serialize for BTreeGraph<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut m = serializer.serialize_map(Some(self.index.len()))?;
        self.iter()
            .try_for_each(|(key, value)| m.serialize_entry(key, value))?;
        m.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V> Deserialize<'de> for BTreeGraph<K, V>
where
    K: Deserialize<'de> + Ord,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GraphVisitor<K, V>(PhantomData<(K, V)>);

        impl<'de, K, V> Visitor<'de> for GraphVisitor<K, V>
        where
            K: Deserialize<'de> + Ord,
            V: Deserialize<'de>,
        {
            type Value = BTreeGraph<K, V>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a map")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (mut graph, mut index) = match map.size_hint() {
                    Some(size) => (Graph::with_capacity(size), BTreeMap::new()),
                    None => (Graph::new(), BTreeMap::new()),
                };

                while let Some((key, value)) = map.next_entry()? {
                    let node = graph.insert(value);
                    index.insert(key, node);
                }

                Ok(BTreeGraph { graph, index })
            }
        }

        deserializer.deserialize_map(GraphVisitor(PhantomData))
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Ord,
{
    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self.entry {
            btree_map::Entry::Vacant(ent) => unsafe {
                ent.insert(self.graph.insert(default()))
                    .try_get_unchecked_mut()
                    .unwrap()
            },
            btree_map::Entry::Occupied(ent) => unsafe {
                ent.get().try_get_unchecked_mut().unwrap()
            },
        }
    }
}
