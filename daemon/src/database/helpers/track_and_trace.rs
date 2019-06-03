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

use super::models::{
    GridPropertyDefinition, NewAssociatedAgent, NewProperty, NewProposal, NewRecord,
    NewReportedValue, NewReporter, Property, ReportedValue, ReportedValueWithReporterAndMetadata,
    Reporter, ReporterWithMetadata,
};
use super::schema::{
    agent, associated_agent, grid_property_definition, property, proposal, record, reported_value,
    reported_value_with_reporter_and_metadata, reporter, reporter_with_metadata,
};
use super::MAX_BLOCK_NUM;
use serde_json::Value as JsonValue;

use diesel::{
    dsl::{insert_into, min, update},
    pg::PgConnection,
    prelude::*,
    result::Error::NotFound,
    QueryResult,
};

pub fn insert_associated_agents(
    conn: &PgConnection,
    agents: &[NewAssociatedAgent],
) -> QueryResult<()> {
    for agent in agents {
        update_associated_agent_end_block_num(
            conn,
            &agent.record_id,
            &agent.agent_id,
            agent.start_block_num,
        )?;
    }

    insert_into(associated_agent::table)
        .values(agents)
        .execute(conn)
        .map(|_| ())
}

pub fn update_associated_agent_end_block_num(
    conn: &PgConnection,
    record_id: &str,
    agent_id: &str,
    current_block_num: i64,
) -> QueryResult<()> {
    update(associated_agent::table)
        .filter(
            associated_agent::record_id
                .eq(record_id)
                .and(associated_agent::agent_id.eq(agent_id))
                .and(associated_agent::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .set(associated_agent::end_block_num.eq(current_block_num))
        .execute(conn)
        .map(|_| ())
}

pub fn insert_properties(conn: &PgConnection, properties: &[NewProperty]) -> QueryResult<()> {
    for property in properties {
        update_property_end_block_num(
            conn,
            &property.name,
            &property.record_id,
            property.start_block_num,
        )?;
    }

    insert_into(property::table)
        .values(properties)
        .execute(conn)
        .map(|_| ())
}

pub fn update_property_end_block_num(
    conn: &PgConnection,
    name: &str,
    record_id: &str,
    current_block_num: i64,
) -> QueryResult<()> {
    update(property::table)
        .filter(
            property::name
                .eq(name)
                .and(property::record_id.eq(record_id))
                .and(property::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .set(property::end_block_num.eq(current_block_num))
        .execute(conn)
        .map(|_| ())
}

pub fn insert_proposals(conn: &PgConnection, proposals: &[NewProposal]) -> QueryResult<()> {
    for proposal in proposals {
        update_proposal_end_block_num(
            conn,
            &proposal.record_id,
            &proposal.receiving_agent,
            proposal.start_block_num,
        )?;
    }

    insert_into(proposal::table)
        .values(proposals)
        .execute(conn)
        .map(|_| ())
}

pub fn update_proposal_end_block_num(
    conn: &PgConnection,
    record_id: &str,
    receiving_agent: &str,
    current_block_num: i64,
) -> QueryResult<()> {
    update(proposal::table)
        .filter(
            proposal::record_id
                .eq(record_id)
                .and(proposal::receiving_agent.eq(receiving_agent))
                .and(proposal::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .set(proposal::end_block_num.eq(current_block_num))
        .execute(conn)
        .map(|_| ())
}

pub fn insert_records(conn: &PgConnection, records: &[NewRecord]) -> QueryResult<()> {
    for record in records {
        update_record_end_block_num(conn, &record.record_id, record.start_block_num)?;
    }

    insert_into(record::table)
        .values(records)
        .execute(conn)
        .map(|_| ())
}

pub fn update_record_end_block_num(
    conn: &PgConnection,
    record_id: &str,
    current_block_num: i64,
) -> QueryResult<()> {
    update(record::table)
        .filter(
            record::record_id
                .eq(record_id)
                .and(record::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .set(record::end_block_num.eq(current_block_num))
        .execute(conn)
        .map(|_| ())
}

pub fn insert_reported_values(conn: &PgConnection, values: &[NewReportedValue]) -> QueryResult<()> {
    for value in values {
        update_reported_value_end_block_num(
            conn,
            &value.property_name,
            &value.record_id,
            value.start_block_num,
        )?;
    }

    insert_into(reported_value::table)
        .values(values)
        .execute(conn)
        .map(|_| ())
}

pub fn update_reported_value_end_block_num(
    conn: &PgConnection,
    property_name: &str,
    record_id: &str,
    current_block_num: i64,
) -> QueryResult<()> {
    update(reported_value::table)
        .filter(
            reported_value::record_id
                .eq(record_id)
                .and(reported_value::property_name.eq(property_name))
                .and(reported_value::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .set(reported_value::end_block_num.eq(current_block_num))
        .execute(conn)
        .map(|_| ())
}

pub fn insert_reporters(conn: &PgConnection, reporters: &[NewReporter]) -> QueryResult<()> {
    for reporter in reporters {
        update_reporter_end_block_num(
            conn,
            &reporter.property_name,
            &reporter.record_id,
            &reporter.public_key,
            reporter.start_block_num,
        )?;
    }

    insert_into(reporter::table)
        .values(reporters)
        .execute(conn)
        .map(|_| ())
}

pub fn update_reporter_end_block_num(
    conn: &PgConnection,
    property_name: &str,
    record_id: &str,
    public_key: &str,
    current_block_num: i64,
) -> QueryResult<()> {
    update(reporter::table)
        .filter(
            reporter::record_id
                .eq(record_id)
                .and(reporter::property_name.eq(property_name))
                .and(reporter::public_key.eq(public_key))
                .and(reporter::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .set(reporter::end_block_num.eq(current_block_num))
        .execute(conn)
        .map(|_| ())
}

pub fn fetch_property(
    conn: &PgConnection,
    record_id: &str,
    property_name: &str,
) -> QueryResult<Option<Property>> {
    property::table
        .filter(
            property::name
                .eq(property_name)
                .and(property::record_id.eq(record_id))
                .and(property::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn fetch_reported_value_with_reporter_and_metadata(
    conn: &PgConnection,
    record_id: &str,
    property_name: &str,
) -> QueryResult<Option<ReportedValueWithReporterAndMetadata>> {
    reported_value_with_reporter_and_metadata::table
        .filter(
            reported_value_with_reporter_and_metadata::property_name
                .eq(property_name)
                .and(reported_value_with_reporter_and_metadata::record_id.eq(record_id))
                .and(
                    reported_value_with_reporter_and_metadata::reported_value_end_block_num
                        .eq(MAX_BLOCK_NUM),
                ),
        )
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn fetch_reporter_with_metadata(
    conn: &PgConnection,
    record_id: &str,
    property_name: &str,
    reporter_index: i32,
) -> QueryResult<Option<ReporterWithMetadata>> {
    reporter_with_metadata::table
        .filter(
            reporter_with_metadata::property_name
                .eq(property_name)
                .and(reporter_with_metadata::record_id.eq(record_id))
                .and(reporter_with_metadata::reporter_index.eq(reporter_index))
                .and(reporter_with_metadata::reporter_end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn list_reporters(
    conn: &PgConnection,
    record_id: &str,
    property_name: &str,
) -> QueryResult<Vec<Reporter>> {
    reporter::table
        .filter(
            reporter::property_name
                .eq(property_name)
                .and(reporter::record_id.eq(record_id))
                .and(reporter::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .load::<Reporter>(conn)
}

pub fn list_reported_value_with_reporter_and_metadata(
    conn: &PgConnection,
    record_id: &str,
    property_name: &str,
) -> QueryResult<Vec<ReportedValueWithReporterAndMetadata>> {
    reported_value_with_reporter_and_metadata::table
        .filter(
            reported_value_with_reporter_and_metadata::property_name
                .eq(property_name)
                .and(reported_value_with_reporter_and_metadata::record_id.eq(record_id))
                .and(
                    reported_value_with_reporter_and_metadata::reported_value_end_block_num
                        .le(MAX_BLOCK_NUM),
                ),
        )
        .load::<ReportedValueWithReporterAndMetadata>(conn)
}
