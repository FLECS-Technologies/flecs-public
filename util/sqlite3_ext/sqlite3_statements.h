// Copyright 2021-2022 FLECS Technologies GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#ifndef FLECS_util_sqlit3_ext_sqlite3_statements_h
#define FLECS_util_sqlit3_ext_sqlite3_statements_h

namespace FLECS {

constexpr const char* insert_stmt = "INSERT INTO %s VALUES (\"";
constexpr const char* insert_or_replace_stmt = "INSERT OR REPLACE INTO %s VALUE(\"";
constexpr const char* delete_stmt = "DELETE FROM %s WHERE ";
constexpr const char* select_all_stmt = "SELECT * FROM %s ";
constexpr const char* create_table_stmt = "CREATE TABLE IF NOT EXISTS %s (";

} // namespace FLECS

#endif // FLECS_util_sqlit3_ext_sqlite3_statements_h
