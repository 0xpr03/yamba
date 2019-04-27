/*
 *  YAMBA manager
 *  Copyright (C) 2019 Aron Heinecke
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */
use crate::models::*;
use yamba_types::models::ID;

use failure::Fallible;

#[cfg(feature = "local")]
mod local;
#[cfg(any(feature = "maria", feature = "postgres"))]
mod remote;

#[cfg(any(feature = "maria", feature = "postgres"))]
pub mod schema;

#[cfg(any(feature = "maria", feature = "postgres"))]
pub use remote::DB;

#[cfg(feature = "local")]
pub use local::DB;

macro_rules! assert_unique_feature {
    () => {};
    ($first:tt $(,$rest:tt)*) => {
        $(
            #[cfg(all(feature = $first, feature = $rest))]
            compile_error!(concat!("features \"", $first, "\" and \"", $rest, "\" cannot be used together"));
        )*
        assert_unique_feature!($($rest),*);
    }
}

assert_unique_feature!("maria", "local", "postgres");

pub type UID = i64; // TODO
pub type Permission = String; // TODO

pub trait Database: Send + Sync + Clone {
    type DB: Database;
    /// Create Database
    fn create(path: String) -> Fallible<DB>;
    /// Get Instance by ID
    fn get_instance(&self, id: ID) -> Fallible<Instance>;
    /// Get all instances, returns only autostart instances when isAutostart has been set
    fn get_instances(&self, is_autostart: bool) -> Fallible<Vec<Instance>>;
    /// Create a new instance
    fn create_instance(&self, instance: NewInstance) -> Fallible<Instance>;
    // /// Get user by UID
    // fn get_user(&self, uid: UID) -> Fallible<User>;
    // /// Create user
    // fn create_user(&self, user: User) -> Fallible<()>;
    // /// Check permission for user
    // fn has_perm(&self, uid: UID, perm: Permission) -> Fallible<bool>;
}
