// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::database::{
    helpers as db,
    models::{
        GridPropertyDefinition, GridSchema, LatLongValue, Property, ReportedValue,
        ReportedValueWithReporterAndMetadata, Reporter, ReporterWithMetadata,
    },
    ConnectionPool,
};
use crate::rest_api::{error::RestApiResponseError, routes::DbExecutor, AppState};
use serde_json::{Map, Value as JsonValue};

use super::schemas::GridPropertyDefinitionSlice;
use actix::{Handler, Message, SyncContext};
use actix_web::{AsyncResponder, HttpRequest, HttpResponse, Path};
use futures::Future;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PropertySlice {
    pub name: String,
    pub record_id: String,
    pub data_type: String,
    pub reporters: Vec<String>,
    pub updates: Vec<PropertyValueSlice>,
    pub value: PropertyValueSlice,
}
impl PropertySlice {
    pub fn from_model(
        property: &Property,
        reporters: &[String],
        data_type: &str,
        updates: &[PropertyValueSlice],
        value: PropertyValueSlice,
    ) -> PropertySlice {
        PropertySlice {
            name: property.name.clone(),
            record_id: property.record_id.clone(),
            data_type: data_type.to_string(),
            reporters: reporters.to_vec(),
            updates: updates.to_vec(),
            value,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PropertyValueSlice {
    pub timestamp: u64,
    pub value: Value,
    pub reporter: ReporterSlice,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Value(
    #[serde(skip_serializing_if = "Option::is_none")] Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")] Option<Vec<PropertyValueSlice>>,
    #[serde(skip_serializing_if = "Option::is_none")] Option<LatLong>,
    #[serde(skip_serializing_if = "Option::is_none")] Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")] Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")] Option<Vec<u8>>,
);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LatLong {
    pub latitude: i64,
    pub longitude: i64,
}

impl LatLong {
    pub fn from_model(lat_long_value: LatLongValue) -> LatLong {
        LatLong {
            latitude: lat_long_value.0,
            longitude: lat_long_value.1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReporterSlice {
    pub public_key: String,
    pub metadata: JsonValue,
}

impl ReporterSlice {
    pub fn from_model(reporter_with_metadata: &ReporterWithMetadata) -> ReporterSlice {
        ReporterSlice {
            public_key: reporter_with_metadata.public_key.clone(),
            metadata: reporter_with_metadata
                .metadata
                .clone()
                .unwrap_or(JsonValue::Object(Map::new())),
        }
    }
}

impl PropertyValueSlice {
    pub fn from_model(
        reported_value_with_reporter: &ReportedValueWithReporterAndMetadata,
        struct_values: Option<Vec<PropertyValueSlice>>,
    ) -> Result<PropertyValueSlice, RestApiResponseError> {
        Ok(PropertyValueSlice {
            timestamp: reported_value_with_reporter.timestamp.clone() as u64,
            value: parse_value(reported_value_with_reporter, struct_values)?,
            reporter: ReporterSlice {
                public_key: reported_value_with_reporter
                    .public_key
                    .clone()
                    .unwrap_or("".to_string()),
                metadata: reported_value_with_reporter
                    .metadata
                    .clone()
                    .unwrap_or(JsonValue::Object(Map::new())),
            },
        })
    }
}

fn parse_value(
    val: &ReportedValueWithReporterAndMetadata,
    struct_values: Option<Vec<PropertyValueSlice>>,
) -> Result<Value, RestApiResponseError> {
    match val.data_type.as_ref() {
        "String" => {
            if val.string_value.is_none() {
                return Err(RestApiResponseError::DatabaseError(
                    "ReportedValue is of String data_type, but is missing string value".to_string(),
                ));
            }
            Ok(Value(
                val.string_value.clone(),
                None,
                None,
                None,
                None,
                None,
                None,
            ))
        }
        "Boolean" => {
            if val.boolean_value.is_none() {
                return Err(RestApiResponseError::DatabaseError(
                    "ReportedValue is of Boolean data_type, but is missing boolean value"
                        .to_string(),
                ));
            }
            Ok(Value(
                None,
                val.boolean_value.clone(),
                None,
                None,
                None,
                None,
                None,
            ))
        }
        "Enum" => {
            if val.enum_value.is_none() {
                return Err(RestApiResponseError::DatabaseError(
                    "ReportedValue is of Enum data_type, but is missing enum value".to_string(),
                ));
            }
            Ok(Value(
                None,
                None,
                None,
                None,
                None,
                val.enum_value.clone(),
                None,
            ))
        }
        "LatLong" => {
            let lat_long = match val.lat_long_value.clone() {
                Some(lat_long_value) => LatLong::from_model(lat_long_value),
                None => {
                    return Err(RestApiResponseError::DatabaseError(
                        "ReportedValue is of LatLong data_type, but is missing lat_long value"
                            .to_string(),
                    ))
                }
            };
            Ok(Value(None, None, None, Some(lat_long), None, None, None))
        }
        "Number" => {
            if val.number_value.is_none() {
                return Err(RestApiResponseError::DatabaseError(
                    "ReportedValue is of Number data_type, but is missing number value".to_string(),
                ));
            }
            Ok(Value(
                None,
                None,
                None,
                None,
                val.number_value.clone(),
                None,
                None,
            ))
        }
        "Bytes" => {
            if val.bytes_value.is_none() {
                return Err(RestApiResponseError::DatabaseError(
                    "ReportedValue is of Bytes data_type, but is missing bytes value".to_string(),
                ));
            }
            Ok(Value(
                None,
                None,
                None,
                None,
                None,
                None,
                val.bytes_value.clone(),
            ))
        }
        "Struct" => {
            if struct_values.is_none() {
                return Err(RestApiResponseError::DatabaseError(
                    "ReportedValue is of Struct data_type, but is missing struct value".to_string(),
                ));
            }
            Ok(Value(None, None, struct_values, None, None, None, None))
        }
        _ => Err(RestApiResponseError::DatabaseError(format!(
            "Invalid data type in ReportedValue: {}",
            val.data_type
        ))),
    }
}

struct FetchRecordProperty {
    record_id: String,
    property_name: String,
}

impl Message for FetchRecordProperty {
    type Result = Result<PropertySlice, RestApiResponseError>;
}

pub fn fetch_record_property(
    req: HttpRequest<AppState>,
    params: Path<(String, String)>,
) -> impl Future<Item = HttpResponse, Error = RestApiResponseError> {
    req.state()
        .database_connection
        .send(FetchRecordProperty {
            record_id: params.0.clone(),
            property_name: params.1.clone(),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(record) => Ok(HttpResponse::Ok().json(record)),
            Err(err) => Err(err),
        })
}

impl Handler<FetchRecordProperty> for DbExecutor {
    type Result = Result<PropertySlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchRecordProperty, _: &mut SyncContext<Self>) -> Self::Result {
        let property = db::fetch_property(
            &*self.connection_pool.get()?,
            &msg.record_id,
            &msg.property_name,
        )?
        .ok_or(RestApiResponseError::NotFoundError(format!(
            "Could not find property {} for record {}",
            msg.property_name, msg.record_id
        )))?;

        let reporters = db::list_reporters(
            &*self.connection_pool.get()?,
            &msg.record_id,
            &msg.property_name,
        )?;

        let reported_value = db::fetch_reported_value_with_reporter_and_metadata(
            &*self.connection_pool.get()?,
            &msg.record_id,
            &msg.property_name,
        )?
        .ok_or(RestApiResponseError::NotFoundError(format!(
            "Could not find values for property {} for record {}",
            msg.property_name, msg.record_id
        )))?;

        let active_reporters = reporters
            .iter()
            .filter_map(|reporter| {
                if (reporter.authorized) {
                    Some(reporter.public_key.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        let property_value_slice = parse_reported_values(
            &self.connection_pool,
            &reported_value.property_name,
            &reported_value,
        )?;

        let updates = db::list_reported_value_with_reporter_and_metadata(
            &*self.connection_pool.get()?,
            &msg.record_id,
            &msg.property_name,
        )?
        .iter()
        .map(|reported_value| {
            parse_reported_values(
                &self.connection_pool,
                &reported_value.property_name,
                &reported_value,
            )
        })
        .collect::<Result<Vec<PropertyValueSlice>, _>>()?;

        let property_info = PropertySlice::from_model(
            &property,
            &active_reporters,
            &reported_value.data_type,
            &updates,
            property_value_slice.clone(),
        );

        println!("property_info {:?}", property_info);
        println!("t {:?}", property_value_slice);
        Ok(property_info)
    }
}

fn parse_reported_values(
    conn: &ConnectionPool,
    property_name: &str,
    reported_value: &ReportedValueWithReporterAndMetadata,
) -> Result<PropertyValueSlice, RestApiResponseError> {
    if reported_value.data_type == "Struct" {
        let mut property_value_slices = vec![];
        let struct_values = reported_value.struct_values.clone().ok_or_else(|| {
            RestApiResponseError::DatabaseError(
                "ReportedValue is of Struct data_type, but is missing struct values".to_string(),
            )
        })?;
        for value_name in struct_values {
            let struct_value = db::fetch_reported_value_with_reporter_and_metadata(
                &*conn.get()?,
                &reported_value.record_id,
                &value_name,
            )?
            .ok_or(RestApiResponseError::NotFoundError(format!(
                "Could not find values for property {} for struct value {} in record {}",
                value_name, property_name, reported_value.record_id
            )))?;

            let struct_property_name = format!("{}_{}", property_name, value_name);
            println!("struct_property_name {:?}", struct_property_name);
            let property_value_slice =
                parse_reported_values(conn, &struct_property_name, &struct_value)?;
            property_value_slices.push(property_value_slice);
        }
        PropertyValueSlice::from_model(reported_value, Some(property_value_slices))
    } else {
        PropertyValueSlice::from_model(reported_value, None)
    }
}

// pub fn test_what_is_needed(query_result: PropertyInfoQueryResult) {
//     let (current_property, outdate_properties): (PropertyInfoQueryResult, PropertyInfoQueryResult) =
//         query_result
//             .into_iter()
//             .partition(|property_info| property_info.0.end_block_num == db::MAX_BLOCK_NUM);
//
//     println!("current_property {:?}", current_property);
//     println!("outdate_properties {:?}", outdate_properties);
// }

// impl From<(Property, ReportedValue, GridPropertyValue, Reporter)> for PropertyInfo {
//     fn from(
//         (property, reported_value, property_value, reporter): (
//             Certificate,
//             Organization,
//             Standard,
//             Organization,
//         ),
//     ) -> Self {
//         ApiCertificate {
//             id: certificate.certificate_id,
//             certifying_body_id: auditor.organization_id,
//             certifying_body: auditor.name,
//             factory_id: factory.organization_id,
//             factory_name: factory.name,
//             standard_id: certificate.standard_id,
//             standard_name: standard.name,
//             standard_version: certificate.standard_version,
//             valid_from: certificate.valid_from,
//             valid_to: certificate.valid_to,
//         }
//     }
// }

// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// TODO Need to add metadata to reported_value_with_reporter query!
