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

use std::{borrow::Borrow, cmp::Ordering, collections::BTreeMap};
#[cfg(feature = "serde")]
use std::{fmt::Formatter, marker::PhantomData};

#[cfg(feature = "serde")]
use serde::{
    de::{Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};
#[cfg(feature = "tsify")]
use tsify::Tsify;

use crate::{BTreeGraph, Graph, IndexBy, OptRefBy, Ref, RefBy};

#[cfg_attr(feature = "tsify", derive(Tsify))]
#[cfg_attr(feature = "tsify", tsify(from_wasm_abi, into_wasm_abi, type = "[K]"))]
pub struct RefMap<K, V>(BTreeMap<K, Ref<V>>);

#[cfg_attr(feature = "tsify", derive(Tsify))]
#[cfg_attr(feature = "tsify", tsify(from_wasm_abi, into_wasm_abi, type = "[K]"))]
pub struct OptRefMap<K, V>(BTreeMap<K, Option<Ref<V>>>);

impl<K, V> RefMap<K, V> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn with_capacity(_n: usize) -> Self {
        // Self(BTreeMap::with_capacity(n))
        Self::new()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_ref<Q>(&self, key: &Q) -> Option<&Ref<V>>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.0.get(key)
    }

    pub fn get_ref_by<Q>(&self, key: &Q) -> Option<RefBy<K, V>>
    where
        K: Borrow<Q> + Ord + Clone,
        Q: Ord,
    {
        let (key, value) = self.0.get_key_value(key)?;
        Some(RefBy::new(key.clone(), value.clone()))
    }

    pub fn get<'a, Q>(&self, key: &Q, graph: &'a BTreeGraph<K, V>) -> Option<&'a V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        Some(graph.borrow(self.get_ref(key)?))
    }

    pub fn get_mut<'a, Q>(&self, key: &Q, graph: &'a mut BTreeGraph<K, V>) -> Option<&'a V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        Some(graph.borrow_mut(self.get_ref(key)?))
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.0.contains_key(key)
    }

    pub fn iter_ref(&self) -> impl Iterator<Item = (&K, &Ref<V>)> {
        self.0.iter()
    }

    pub fn iter_ref_by(&self) -> impl Iterator<Item = RefBy<K, V>> + '_
    where
        K: Clone,
    {
        self.iter_ref()
            .map(|(k, v)| RefBy::new(k.clone(), v.clone()))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.keys()
    }

    pub fn value_refs(&self) -> impl Iterator<Item = &Ref<V>> {
        self.0.values()
    }

    pub fn iter<'a, G: AsRef<Graph<V>>>(
        &'a self,
        graph: &'a G,
    ) -> impl Iterator<Item = (&'a K, &'a V)> {
        let graph = graph.as_ref();
        self.iter_ref().map(|(k, v)| (k, graph.borrow(v)))
    }

    // pub fn iter_mut<'a, Q>(
    //     &'a self,
    //     graph: &'a mut BTreeGraph<Q, V>,
    // ) -> impl Iterator<Item = (&'a K, &'a mut V)> {
    //     self.iter_ref().map(|(k, v)| (k, graph.borrow_mut(v)))
    // }

    pub fn values<'a, G: AsRef<Graph<V>>>(
        &'a self,
        graph: &'a G,
    ) -> impl Iterator<Item = &'a V> + 'a {
        let graph = graph.as_ref();
        self.value_refs().map(|v| graph.borrow(v))
    }

    pub fn insert(&mut self, key: K, value: Ref<V>) -> Option<Ref<V>>
    where
        K: Ord,
    {
        self.0.insert(key, value)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<Ref<V>>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.0.remove(key)
    }

    pub fn append(&mut self, other: &mut Self)
    where
        K: Ord,
    {
        self.0.append(&mut other.0)
    }

    pub fn resolve<I>(&mut self, index: &I) -> Result<(), K>
    where
        K: Ord + Clone,
        I: IndexBy<K, V>,
    {
        self.0
            .iter_mut()
            .try_for_each(|(key, value)| match index.get(key) {
                Some(v) => {
                    *value = v.clone();
                    Ok(())
                }
                None => Err(key.clone()),
            })
    }
}

impl<K, V> Default for RefMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialEq, V> PartialEq for RefMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len() && self.0.keys().zip(other.0.keys()).all(|(a, b)| a == b)
    }
}

impl<K: Eq, V> Eq for RefMap<K, V> {}

impl<K: PartialOrd, V> PartialOrd for RefMap<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0
            .keys()
            .zip(other.keys())
            .find_map(|(a, b)| match a.partial_cmp(b) {
                Some(Ordering::Equal) => None,
                x => Some(x),
            })
            .unwrap_or_else(|| self.0.len().partial_cmp(&other.0.len()))
    }
}

impl<K: Ord, V> Ord for RefMap<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .keys()
            .zip(other.keys())
            .find_map(|(a, b)| match a.cmp(b) {
                Ordering::Equal => None,
                x => Some(x),
            })
            .unwrap_or_else(|| self.0.len().cmp(&other.0.len()))
    }
}

impl<K, V> IntoIterator for RefMap<K, V> {
    type Item = (K, Ref<V>);
    type IntoIter = std::collections::btree_map::IntoIter<K, Ref<V>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K, V> FromIterator<(K, Ref<V>)> for RefMap<K, V>
where
    K: Ord,
{
    fn from_iter<T: IntoIterator<Item = (K, Ref<V>)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(feature = "serde")]
impl<K, V> Serialize for RefMap<K, V>
where
    K: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(Some(self.0.len()))?;
        self.0.keys().try_for_each(|key| s.serialize_element(key))?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V> Deserialize<'de> for RefMap<K, V>
where
    K: Deserialize<'de> + Ord,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SeqVisitor<K, V>(PhantomData<(K, V)>);

        impl<'de, K, V> Visitor<'de> for SeqVisitor<K, V>
        where
            K: Deserialize<'de> + Ord,
        {
            type Value = RefMap<K, V>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a sequence of keys")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut map = match seq.size_hint() {
                    Some(size) => RefMap::with_capacity(size),
                    None => RefMap::new(),
                };

                while let Some(node) = seq.next_element()? {
                    map.insert(node, Ref::dangling());
                }

                Ok(map)
            }
        }

        deserializer.deserialize_seq(SeqVisitor(PhantomData))
    }
}

impl<K: Clone, V> Clone for RefMap<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K, V> OptRefMap<K, V> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn with_capacity(_n: usize) -> Self {
        // Self(BTreeMap::with_capacity(n))
        Self::new()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_ref<Q>(&self, key: &Q) -> Option<&Ref<V>>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.0.get(key)?.as_ref()
    }

    pub fn get_ref_by<Q>(&self, key: &Q) -> Option<RefBy<K, V>>
    where
        K: Borrow<Q> + Ord + Clone,
        Q: Ord,
    {
        let (key, value) = self.0.get_key_value(key)?;
        let value = value.as_ref()?;
        Some(RefBy::new(key.clone(), value.clone()))
    }

    pub fn get<'a, Q>(&self, key: &Q, graph: &'a BTreeGraph<K, V>) -> Option<&'a V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        Some(graph.borrow(self.get_ref(key)?))
    }

    pub fn get_mut<'a, Q>(&self, key: &Q, graph: &'a mut BTreeGraph<K, V>) -> Option<&'a V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        Some(graph.borrow_mut(self.get_ref(key)?))
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.0.contains_key(key)
    }

    pub fn iter_ref(&self) -> impl Iterator<Item = (&K, &Option<Ref<V>>)> {
        self.0.iter()
    }

    pub fn iter_ref_by(&self) -> impl Iterator<Item = OptRefBy<K, V>> + '_
    where
        K: Clone,
    {
        self.iter_ref()
            .map(|(k, v)| OptRefBy::new(k.clone(), v.clone()))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.keys()
    }

    pub fn value_refs(&self) -> impl Iterator<Item = &Option<Ref<V>>> {
        self.0.values()
    }

    pub fn iter<'a, Q>(
        &'a self,
        graph: &'a BTreeGraph<Q, V>,
    ) -> impl Iterator<Item = (&'a K, Option<&'a V>)> {
        self.iter_ref()
            .map(|(k, v)| (k, v.as_ref().map(|v| graph.borrow(v))))
    }

    // pub fn iter_mut<'a, Q>(
    //     &'a self,
    //     graph: &'a mut BTreeGraph<Q, V>,
    // ) -> impl Iterator<Item = (&'a K, &'a mut V)> {
    //     self.iter_ref().map(|(k, v)| (k, graph.borrow_mut(v)))
    // }

    pub fn values<'a, Q>(
        &'a self,
        graph: &'a BTreeGraph<Q, V>,
    ) -> impl Iterator<Item = &'a V> + 'a {
        self.value_refs()
            .filter_map(|v| Some(graph.borrow(v.as_ref()?)))
    }

    pub fn insert(&mut self, key: K, value: Option<Ref<V>>)
    where
        K: Ord,
    {
        self.0.insert(key, value);
    }

    pub fn remove<Q>(&mut self, key: &Q)
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.0.remove(key);
    }

    pub fn resolve<I>(&mut self, index: &I)
    where
        K: Ord + Clone,
        I: IndexBy<K, V>,
    {
        self.0
            .iter_mut()
            .for_each(|(key, value)| *value = index.get(key).cloned())
    }
}

impl<K, V> Default for OptRefMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> IntoIterator for OptRefMap<K, V> {
    type Item = (K, Option<Ref<V>>);
    type IntoIter = std::collections::btree_map::IntoIter<K, Option<Ref<V>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K, V> FromIterator<(K, Option<Ref<V>>)> for OptRefMap<K, V>
where
    K: Ord,
{
    fn from_iter<T: IntoIterator<Item = (K, Option<Ref<V>>)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(feature = "serde")]
impl<K, V> Serialize for OptRefMap<K, V>
where
    K: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(Some(self.0.len()))?;
        self.0.keys().try_for_each(|key| s.serialize_element(key))?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V> Deserialize<'de> for OptRefMap<K, V>
where
    K: Deserialize<'de> + Ord,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SeqVisitor<K, V>(PhantomData<(K, V)>);

        impl<'de, K, V> Visitor<'de> for SeqVisitor<K, V>
        where
            K: Deserialize<'de> + Ord,
        {
            type Value = OptRefMap<K, V>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a sequence of keys")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut map = match seq.size_hint() {
                    Some(size) => OptRefMap::with_capacity(size),
                    None => OptRefMap::new(),
                };

                while let Some(node) = seq.next_element()? {
                    map.insert(node, None);
                }

                Ok(map)
            }
        }

        deserializer.deserialize_seq(SeqVisitor(PhantomData))
    }
}

impl<K: Clone, V> Clone for OptRefMap<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
