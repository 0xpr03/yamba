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

use serde::Deserialize;

use yamba_types::ID;

#[derive(Debug, Deserialize)]
pub struct TSCreate {
    pub host: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub cid: Option<i32>,
    #[serde(default)]
    pub password: Option<String>,
    pub id: ID,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use serde_urlencoded;
    #[test]
    fn parse_form() {
        let _v: TSCreate = serde_json::from_str(r#" {"id":1, "name":"myYAMBAInstance", "host":"ek.proctet.net", "password":"", "port":null, "cid":5369 }"#).unwrap();

        let v = serde_urlencoded::from_str::<TSCreate>(
            r#"id=1&name=myYAMBAInstance&host=ek.proctet.net&password=&port=&cid=5369"#,
        );
        if let Err(ref e) = v {
            println!("{}", e);
        }

        v.unwrap();
    }
}
