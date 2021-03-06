/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use serde_json::Value as JsonValue;

use super::schema::{agent, block, organization};

#[derive(Insertable, Queryable)]
#[table_name = "block"]
pub struct Block {
    pub block_id: String,
    pub block_num: i64,
    pub state_root_hash: String,
}

#[derive(Insertable, Debug)]
#[table_name = "agent"]
pub struct NewAgent {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub metadata: Vec<JsonValue>,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_block_num: i64,
    pub end_block_num: i64,
}

#[derive(Queryable, Debug)]
pub struct Agent {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub metadata: Vec<JsonValue>,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_block_num: i64,
    pub end_block_num: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "organization"]
pub struct NewOrganization {
    pub org_id: String,
    pub name: String,
    pub address: String,
    pub metadata: Vec<JsonValue>,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_block_num: i64,
    pub end_block_num: i64,
}

#[derive(Queryable, Debug)]
pub struct Organization {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub org_id: String,
    pub name: String,
    pub address: String,
    pub metadata: Vec<JsonValue>,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_block_num: i64,
    pub end_block_num: i64,
}
