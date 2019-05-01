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
use super::Database;
use crate::models::*;
use diesel::*;
use failure::Fallible;
use yamba_types::models::ID;

pub struct DB {}

impl Database for DB {
    type DB = DB;
    fn create(path: String) -> Fallible<DB> {
        unimplemented!();
    }
    fn get_instance(&self, id: ID) -> Fallible<Instance> {
        unimplemented!();
    }
    fn get_instances(&self) -> Fallible<Vec<Instance>> {
        unimplemented!();
    }
    fn create_instance(&self, instance: NewInstance) -> Fallible<Instance> {
        unimplemented!();
    }
}
