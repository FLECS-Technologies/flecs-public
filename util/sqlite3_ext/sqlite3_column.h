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

#ifndef FLECS_util_sqlite3_column_h
#define FLECS_util_sqlite3_column_h

#include <string>

#include "sqlite3.h"
#include "util/string/string_utils.h"

namespace FLECS {

class sqlite3_column_t
{
public:
    sqlite3_column_t(std::string name, int sqlite3_type, std::size_t width = 0)
        : _name{name}
        , _type{}
    {
        switch (sqlite3_type)
        {
            case SQLITE_INTEGER: {
                _type = "INTEGER";
                break;
            }
            case SQLITE_FLOAT: {
                _type = "REAL";
                break;
            }
            case SQLITE_TEXT: {
                _type += "TEXT(" + stringify(width) + ")";
                break;
            }
            case SQLITE_BLOB: {
                _type += "BLOB";
                break;
            }
            case SQLITE_NULL: {
                break;
            }
            default: {
                break;
            }
        }
    }

    const std::string& name() const noexcept { return _name; }
    const std::string& type() const noexcept { return _type; }

private:
    std::string _name;
    std::string _type;
};

inline std::string to_string(const sqlite3_column_t& col)
{
    return col.name() + ' ' + col.type();
}

} // namespace FLECS

#endif // FLECS_util_sqlite3_column_h
