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

table! {
    agent (id) {
        id -> Int8,
        public_key -> Varchar,
        org_id -> Varchar,
        active -> Bool,
        roles -> Array<Varchar>,
        metadata -> Array<Json>,
        start_block_num -> Int8,
        end_block_num -> Int8,
    }
}

table! {
    block (block_id) {
        block_id -> Varchar,
        block_num -> Int8,
        state_root_hash -> Varchar,
    }
}

table! {
    chain_record (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
    }
}

table! {
    organization (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
        org_id -> Varchar,
        name -> Varchar,
        address -> Varchar,
        metadata -> Array<Json>,
    }
}

allow_tables_to_appear_in_same_query!(agent, block, chain_record, organization,);
