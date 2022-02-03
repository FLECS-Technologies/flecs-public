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

#ifndef FLECS_util_sqlite3_table_h
#define FLECS_util_sqlite3_table_h

#include <cstdint>

namespace FLECS {

struct sqlite3_type_t
{
    const char* type_str;
    std::size_t len;
};

struct sqlite3_table_field_t
{
};

} // namespace FLECS

#endif // FLECS_util_sqlite3_table_h
