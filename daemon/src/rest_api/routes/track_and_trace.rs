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
        GridPropertyDefinition, GridPropertyValue, GridSchema, LatLongValue, Property,
        ReportedValue, Reporter,
    },
};
use crate::rest_api::{error::RestApiResponseError, routes::DbExecutor, AppState};

use super::schemas::GridPropertyDefinitionSlice;
use actix::{Handler, Message, SyncContext};
use actix_web::{AsyncResponder, HttpRequest, HttpResponse, Path};
use futures::Future;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type PropertyInfoQueryResult = Vec<(
    Property,
    Option<GridPropertyDefinition>,
    Option<ReportedValue>,
    Option<GridPropertyValue>,
    Option<Reporter>,
)>;

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyInfo {
    pub name: String,
    pub record_id: String,
    pub property_definition: GridPropertyDefinitionSlice,
    pub reporters: Vec<String>,
    pub updates: Vec<PropertyValueSlice>,
    pub value: PropertyValueSlice,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyValueSlice {
    pub timestamp: u64,
    pub data_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boolean_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub struct_values: Option<Vec<PropertyValueSlice>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat_long_value: Option<(i64, i64)>,
    pub reporters: Vec<ReporterSlice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReporterSlice {
    pub name: u64,
    pub public_key: String,
}

pub fn test_what_is_needed(query_result: PropertyInfoQueryResult) {
    let (current_property, outdate_properties): (PropertyInfoQueryResult, PropertyInfoQueryResult) =
        query_result
            .into_iter()
            .partition(|property_info| property_info.0.end_block_num == db::MAX_BLOCK_NUM);

    println!("current_property {:?}", current_property);
    println!("outdate_properties {:?}", outdate_properties);
}

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
