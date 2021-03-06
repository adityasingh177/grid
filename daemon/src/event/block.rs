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

use diesel::prelude::*;
use diesel::result::Error;
use grid_sdk::{
    protocol::pike::state::{AgentList, OrganizationList},
    protos::FromBytes,
};
use protobuf;
use sawtooth_sdk::messages::{
    events::Event,
    events::Event_Attribute,
    transaction_receipt::{StateChange, StateChangeList},
};
use serde_json::Value as JsonValue;

use crate::database::{
    helpers as db,
    models::{Block, NewAgent, NewOrganization},
    ConnectionPool,
};

use super::{error::EventError, EventHandler, GRID_NAMESPACE, PIKE_NAMESPACE};

pub struct BlockEventHandler {
    connection_pool: ConnectionPool,
}

impl BlockEventHandler {
    pub fn new(connection_pool: ConnectionPool) -> Self {
        Self { connection_pool }
    }
}

impl EventHandler for BlockEventHandler {
    fn handle_events(&self, events: &[Event]) -> Result<(), EventError> {
        let block = get_block(events)?;
        let db_ops = get_db_operations(events, block.block_num)?;

        debug!(
            "Received sawtooth/block-commit ({}, {}, {})",
            block.block_id, block.block_num, block.state_root_hash
        );

        let conn = self
            .connection_pool
            .get()
            .map_err(|err| EventError(format!("Unable to connect to database: {}", err)))?;

        conn.build_transaction()
            .run::<_, Error, _>(|| {
                match db::get_block_by_block_num(&conn, block.block_num) {
                    Ok(Some(ref b)) if b.block_id != block.block_id => {
                        db::resolve_fork(&conn, block.block_num)?;
                        info!(
                            "Fork detected. Replaced {} at height {}, with block {}.",
                            &b.block_id, &b.block_num, &block.block_id
                        );
                    }
                    Ok(Some(_)) => {
                        info!(
                            "Block {} at height {} is duplicate no action taken",
                            &block.block_id, block.block_num
                        );
                    }
                    Ok(None) => info!("Received new block {}", block.block_id),
                    Err(err) => {
                        return Err(err);
                    }
                }

                db_ops
                    .iter()
                    .try_for_each(|op| op.execute(&conn))
                    .and_then(|_| db::insert_block(&conn, &block))
            })
            .map_err(|err| EventError(format!("Database transaction failed {}", err)))
    }
}

fn get_block(events: &[Event]) -> Result<Block, EventError> {
    events
        .into_iter()
        .filter(|event| event.get_event_type() == "sawtooth/block-commit")
        .map(|event| {
            let attributes = event.get_attributes();
            Ok(Block {
                block_id: require_attr(attributes, "block_id")?,
                block_num: require_attr(attributes, "block_num")?
                    .parse::<i64>()
                    .map_err(|err| {
                        EventError(format!("block_num was not a valid number: {}", err))
                    })?,
                state_root_hash: require_attr(attributes, "state_root_hash")?,
            })
        })
        .last()
        .unwrap_or_else(|| Err(EventError("No block found".into())))
}

fn require_attr(attributes: &[Event_Attribute], key: &str) -> Result<String, EventError> {
    attributes
        .iter()
        .find(|attr| attr.get_key() == key)
        .map(|attr| attr.get_value().to_owned())
        .ok_or_else(|| EventError(format!("Unable to find {}", key)))
}

fn get_db_operations(events: &[Event], block_num: i64) -> Result<Vec<DbOperation>, EventError> {
    events
        .into_iter()
        .filter(|event| event.get_event_type() == "sawtooth/state-delta")
        .filter_map(|event| protobuf::parse_from_bytes::<StateChangeList>(&event.data).ok())
        .flat_map(|mut state_changes| state_changes.take_state_changes().into_iter())
        .filter(|state_change| {
            &state_change.address[0..6] == PIKE_NAMESPACE
                || &state_change.address[0..6] == GRID_NAMESPACE
        })
        .map(|state_change| state_change_to_db_operation(&state_change, block_num))
        .collect::<Result<Vec<DbOperation>, EventError>>()
}

fn state_change_to_db_operation(
    state_change: &StateChange,
    block_num: i64,
) -> Result<DbOperation, EventError> {
    match &state_change.address[0..8] {
        "cad11d00" => {
            let agents = AgentList::from_bytes(&state_change.value)
                .map_err(|err| EventError(format!("Failed to parse agent list {}", err)))?
                .agents()
                .into_iter()
                .map(|agent| NewAgent {
                    public_key: agent.public_key().to_string(),
                    org_id: agent.org_id().to_string(),
                    active: *agent.active(),
                    roles: agent.roles().to_vec(),
                    metadata: agent
                        .metadata()
                        .iter()
                        .map(|md| {
                            json!({
                                md.key(): md.value()
                            })
                        })
                        .collect::<Vec<JsonValue>>(),
                    start_block_num: block_num,
                    end_block_num: db::MAX_BLOCK_NUM,
                })
                .collect::<Vec<NewAgent>>();

            Ok(DbOperation::InsertAgents(agents))
        }
        "cad11d01" => {
            let orgs = OrganizationList::from_bytes(&state_change.value)
                .map_err(|err| EventError(format!("Failed to parse organization list {}", err)))?
                .organizations()
                .into_iter()
                .map(|org| NewOrganization {
                    org_id: org.org_id().to_string(),
                    name: org.name().to_string(),
                    address: org.address().to_string(),
                    metadata: org
                        .metadata()
                        .iter()
                        .map(|md| {
                            json!({
                                md.key(): md.value()
                            })
                        })
                        .collect::<Vec<JsonValue>>(),
                    start_block_num: block_num,
                    end_block_num: db::MAX_BLOCK_NUM,
                })
                .collect::<Vec<NewOrganization>>();

            Ok(DbOperation::InsertOrganizations(orgs))
        }
        _ => Err(EventError(format!(
            "Could not handle state change unknown address: {}",
            &state_change.address
        ))),
    }
}

enum DbOperation {
    InsertAgents(Vec<NewAgent>),
    InsertOrganizations(Vec<NewOrganization>),
}

impl DbOperation {
    fn execute(&self, conn: &PgConnection) -> QueryResult<()> {
        match *self {
            DbOperation::InsertAgents(ref agents) => db::insert_agents(conn, agents),
            DbOperation::InsertOrganizations(ref orgs) => db::insert_organizations(conn, orgs),
        }
    }
}
