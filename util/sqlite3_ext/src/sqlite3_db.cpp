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

#include "sqlite3_db.h"

#include <filesystem>
#include <iostream>

namespace FLECS {

sqlite3_db_t::sqlite3_db_t(const char* filename, int flags, const char* zVfs)
    : _ok{}
    , _db{nullptr}
{
    open(filename, flags, zVfs);
}

sqlite3_db_t::~sqlite3_db_t()
{
    close();
}

int sqlite3_db_t::open(const char* filename, int flags, const char* zVfs)
{
    const auto file_path = std::filesystem::path{filename};
    const auto dir = file_path.parent_path();
    auto ec = std::error_code{};
    std::filesystem::create_directories(dir, ec);
    int res = sqlite3_open_v2(filename, &_db, flags, zVfs);
    if (res != SQLITE_OK)
    {
        std::fprintf(stderr, "Could not open SQLite db %s: %d\n", filename, res);
    }
    _ok = (res == SQLITE_OK);
    return res;
}

int sqlite3_db_t::select_all(const char* table, select_callback_t cbk, void* cbk_arg)
{
    const auto len = std::snprintf(nullptr, 0, select_all_stmt, table);

    auto select_str = std::make_unique<char[]>(len + 1);
    std::snprintf(select_str.get(), len + 1, select_all_stmt, table);

    return exec(select_str.get(), cbk, cbk_arg);
}

int sqlite3_db_t::exec(const char* sql, int (*callback)(void*, int, char**, char**), void* arg)
{
    if (!_ok)
    {
        return SQLITE_ERROR;
    }
    char* errmsg = nullptr;
    auto res = sqlite3_exec(_db, sql, callback, arg, &errmsg);
    if (res != SQLITE_OK)
    {
        std::cerr << "Could not execute sql " << sql << ": " << res << " (" << errmsg << ")" << std::endl;
        sqlite3_free(errmsg);
    }
    return res;
}

int sqlite3_db_t::close()
{
    _ok = false;
    return sqlite3_close_v2(_db);
}

} // namespace FLECS
