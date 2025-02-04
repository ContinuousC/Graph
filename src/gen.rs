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

#[cfg(any(not(feature = "unsafe"), debug_assertions))]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(any(not(feature = "unsafe"), debug_assertions))]
static GENERATION: AtomicU64 = AtomicU64::new(1);

#[cfg(any(not(feature = "unsafe"), debug_assertions))]
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Copy, Hash, Debug)]
pub struct Gen(u64);

#[cfg(any(not(feature = "unsafe"), debug_assertions))]
impl Gen {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(GENERATION.fetch_add(1, Ordering::Relaxed))
    }

    pub fn invalid() -> Self {
        Self(0)
    }

    pub fn is_invalid(&self) -> bool {
        self.0 == 0
    }
}

#[cfg(any(not(feature = "unsafe"), debug_assertions))]
impl PartialEq for Gen {
    fn eq(&self, other: &Self) -> bool {
        self.0 != 0 && self.0 == other.0
    }
}

/* Disable gen checking in optimized "unsafe" build. */

#[cfg(all(feature = "unsafe", not(debug_assertions)))]
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
pub struct Gen;

#[cfg(all(feature = "unsafe", not(debug_assertions)))]
impl Gen {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn invalid() -> Self {
        Self
    }

    pub fn is_invalid(&self) -> bool {
        false
    }
}
