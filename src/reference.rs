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

use std::{hash::Hash, ptr::NonNull};

#[cfg(feature = "serde")]
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
#[cfg(feature = "tsify")]
use tsify::Tsify;

use crate::{Gen, IndexBy};

/// A reference to a graph node.
#[derive(Debug)]
pub struct Ref<T> {
    value: NonNull<Option<T>>,
    pub(crate) gen: Gen,
}

/// A reference with an associated key. This can be used to make a
/// structure serializable.
#[derive(Debug)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[cfg_attr(feature = "tsify", tsify(from_wasm_abi, into_wasm_abi, type = "K"))]
pub struct RefBy<K, V> {
    key: K,
    value: Ref<V>,
}

/// A reference that may or may not be resolvable.
#[derive(Debug)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[cfg_attr(feature = "tsify", tsify(from_wasm_abi, into_wasm_abi, type = "K"))]
pub struct OptRefBy<K, V> {
    key: K,
    value: Option<Ref<V>>,
}

impl<T> Ref<T> {
    pub(crate) fn new(value: NonNull<Option<T>>, gen: Gen) -> Self {
        Self { value, gen }
    }

    /// Create a dangling reference. Trying to borrow its value from a
    /// graph using a safe method will always panic since the validity
    /// check will always fail. Accessing the value using unsafe
    /// functions may cause undefined behavior.
    pub fn dangling() -> Self {
        Self {
            value: NonNull::dangling(),
            gen: Gen::invalid(),
        }
    }

    pub fn is_invalid(&self) -> bool {
        self.gen.is_invalid()
    }

    /// Safety: when using this method, take (at least) a shared
    /// reference to the container and check the Ref's validity
    /// (ref.gen == container.gen).
    pub(crate) unsafe fn try_get_unchecked<'a>(&self) -> Option<&'a T> {
        (*self.value.as_ptr()).as_ref()
    }

    /// Safety: when using this method, take a mutable reference to
    /// the container and check the Ref's validity (ref.gen ==
    /// container.gen).
    pub(crate) unsafe fn try_get_unchecked_mut<'a>(&self) -> Option<&'a mut T> {
        (*self.value.as_ptr()).as_mut()
    }

    /// Safety: when using this method, take a mutable reference to
    /// the container and check the Ref's validity (ref.gen ==
    /// container.gen).
    pub(crate) unsafe fn try_remove_unchecked(&self) -> Option<T> {
        (*self.value.as_ptr()).take()
    }

    /// Safety: when using this method, take a mutable reference to
    /// the container and check the Ref's validity (ref.gen ==
    /// container.gen).
    pub(crate) unsafe fn try_replace_unchecked(&self, value: T) -> Option<T> {
        (*self.value.as_ptr()).replace(value)
    }
}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            gen: self.gen,
        }
    }
}

impl<T> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.gen == other.gen && self.value == other.value
    }
}

impl<T> AsRef<Ref<T>> for Ref<T> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<K, V> RefBy<K, V> {
    pub fn new(key: K, value: Ref<V>) -> Self {
        Self { key, value }
    }

    pub fn dangling(key: K) -> Self {
        Self::new(key, Ref::dangling())
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn value_ref(&self) -> &Ref<V> {
        &self.value
    }

    pub fn pair(&self) -> (&K, &Ref<V>) {
        (&self.key, &self.value)
    }

    pub fn resolve<I>(&mut self, index: &I) -> Result<(), K>
    where
        K: Ord + Clone,
        I: IndexBy<K, V>,
    {
        match index.get(&self.key) {
            Some(v) => {
                self.value = v.clone();
                Ok(())
            }
            None => Err(self.key.clone()),
        }
    }
}

impl<K: Clone, V> Clone for RefBy<K, V> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

impl<K: PartialEq, V> PartialEq for RefBy<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key //&& self.value == other.value
    }
}

impl<K: Eq, V> Eq for RefBy<K, V> {}

impl<K: PartialOrd, V> PartialOrd for RefBy<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<K: Ord, V> Ord for RefBy<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K: Hash, V> Hash for RefBy<K, V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl<K, V> AsRef<Ref<V>> for RefBy<K, V> {
    fn as_ref(&self) -> &Ref<V> {
        &self.value
    }
}

#[cfg(feature = "serde")]
impl<K, V> Serialize for RefBy<K, V>
where
    K: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.key.serialize(serializer)
    }
}

/// Deserialize is implemented if K can be deserialized, by producing
/// dangling references. To be useful, deserialization must be
/// followed by a reference resolution step. Otherwise, any attempt to
/// access the reference's target will result in a panic.
#[cfg(feature = "serde")]
impl<'de, K, V> Deserialize<'de> for RefBy<K, V>
where
    K: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key = K::deserialize(deserializer)?;
        Ok(Self {
            key,
            value: Ref::dangling(),
        })
    }
}

impl<K, V> OptRefBy<K, V> {
    pub fn new(key: K, value: Option<Ref<V>>) -> Self {
        Self { key, value }
    }

    pub fn dangling(key: K) -> Self {
        Self {
            key,
            value: Some(Ref::dangling()),
        }
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn value_ref(&self) -> Option<&Ref<V>> {
        self.value.as_ref()
    }

    pub fn resolve<I>(&mut self, index: &I)
    where
        K: Ord + Clone,
        I: IndexBy<K, V>,
    {
        self.value = index.get(&self.key).cloned();
    }
}

impl<K: Clone, V> Clone for OptRefBy<K, V> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

#[cfg(feature = "serde")]
impl<K, V> Serialize for OptRefBy<K, V>
where
    K: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.key.serialize(serializer)
    }
}

/// Deserialize is implemented if K can be deserialized, by producing
/// unresolved references. To be useful, deserialization must be
/// followed by a reference resolution step.
#[cfg(feature = "serde")]
impl<'de, K, V> Deserialize<'de> for OptRefBy<K, V>
where
    K: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key = K::deserialize(deserializer)?;
        Ok(Self { key, value: None })
    }
}
