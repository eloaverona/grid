-- Copyright 2019 Cargill Incorporated
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.
-- ------------------------------------------------------------------------------

--- This view queries the database and joins the reported value table with the
--- correspondent grid property value and reporter at that block height.
--- It perfoms a inner join between  reported_value, grid_property_value and
--- reporter table and then uses an window function to identify the valid row
--- at each block height.

CREATE VIEW reported_value_with_grid_property_value_and_reporter
AS
  SELECT DISTINCT property_name,
                  record_id,
                  reporter_index,
                  timestamp,
                  value_name,
                  data_type,
                  bytes_value,
                  boolean_value,
                  number_value,
                  string_value,
                  enum_value,
                  struct_values,
                  lat_long_value,
                  end_block_num,
                  public_key,
                  authorized
  FROM   (SELECT DISTINCT Row_number()
                            OVER (
                              partition BY id
                              ORDER BY end_block_num) AS RowNum,
                          *
          FROM   (SELECT reported_value.id,
                         reported_value.property_name,
                         reported_value.record_id,
                         reported_value.reporter_index,
                         reported_value.timestamp,
                         reported_value.value_name,
                         grid_property_value.data_type,
                         grid_property_value.bytes_value,
                         grid_property_value.boolean_value,
                         grid_property_value.number_value,
                         grid_property_value.string_value,
                         grid_property_value.enum_value,
                         grid_property_value.struct_values,
                         grid_property_value.lat_long_value,
                         reporter.end_block_num,
                         reporter.public_key,
                         reporter.authorized
                  FROM   reported_value
                         INNER JOIN grid_property_value
                                ON reported_value.value_name =
                                   grid_property_value.NAME
                                   AND reported_value.start_block_num =
                                       grid_property_value.start_block_num
                                   AND reported_value.end_block_num =
                                       grid_property_value.end_block_num
                         INNER JOIN reporter
                                ON reported_value.record_id = reporter.record_id
                                   AND
                         reported_value.property_name = reporter.property_name
                                   AND
                         reported_value.reporter_index = reporter.reporter_index
                                   AND
                         reported_value.end_block_num <= reporter.end_block_num)
                 AS
                 join_tables) X
  WHERE  rownum = 1;
