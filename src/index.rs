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

use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};

use crate::{BTreeGraph, Ref};

pub trait IndexBy<K, V> {
    fn get(&self, key: &K) -> Option<&Ref<V>>;
}

impl<K, V> IndexBy<K, V> for BTreeGraph<K, V>
where
    K: Ord,
{
    fn get(&self, key: &K) -> Option<&Ref<V>> {
        self.index().get(key)
    }
}

impl<K, V> IndexBy<K, V> for BTreeMap<K, Ref<V>>
where
    K: Ord,
{
    fn get(&self, key: &K) -> Option<&Ref<V>> {
        self.get(key)
    }
}

impl<K, V, S> IndexBy<K, V> for HashMap<K, Ref<V>, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    fn get(&self, key: &K) -> Option<&Ref<V>> {
        self.get(key)
    }
}
