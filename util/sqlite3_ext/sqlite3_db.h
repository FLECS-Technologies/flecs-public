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

#ifndef A46C0241_A55D_44A7_AAD0_70A31A41AFEA
#define A46C0241_A55D_44A7_AAD0_70A31A41AFEA

#include <array>
#include <cstdio>
#include <iostream>
#include <memory>
#include <string>

#include "sqlite3.h"
#include "sqlite3_column.h"
#include "sqlite3_primary.h"
#include "sqlite3_statements.h"
#include "util/string/string_utils.h"

namespace FLECS {

template <typename T>
auto value_to_string(T&& value)
{
    using basic_type_t = std::remove_cv_t<std::remove_reference_t<T>>;

    auto str = std::string{"'"};
    if constexpr (
        std::is_same_v<int, basic_type_t> || std::is_same_v<long, basic_type_t> ||
        std::is_same_v<long long, basic_type_t> || std::is_same_v<unsigned, basic_type_t> ||
        std::is_same_v<unsigned long, basic_type_t> || std::is_same_v<unsigned long long, basic_type_t> ||
        std::is_same_v<float, basic_type_t> || std::is_same_v<double, basic_type_t> ||
        std::is_same_v<long double, basic_type_t>)
    {
        using std::to_string;
        str += to_string(value);
    }
    else if constexpr (std::is_enum_v<basic_type_t>)
    {
        str += static_cast<std::underlying_type_t<basic_type_t>>(value);
    }
    else
    {
        str += value;
    }
    str += "'";
    return str;
}

template <typename Arg>
auto format_where_impl(std::string where, const char* const* condition, Arg&& arg)
{
    where += *condition;
    where += "=";
    where += value_to_string(arg);
    where += ";";
    return where;
}

template <typename Arg, typename... Args>
auto format_where_impl(std::string where, const char* const* condition, Arg&& arg, Args&&... args)
{
    where += *condition;
    where += "=";
    where += value_to_string(arg);
    where += " AND ";
    return format_where_impl(where, condition + 1, args...);
}

template <typename... Args>
auto format_where(const char* const* condition, Args&&... args)
{
    const auto where = std::string{" WHERE "};
    return format_where_impl(where, condition, args...);
}

class sqlite3_db_t
{
public:
    const char* errmsg() const noexcept { return sqlite3_errmsg(_db); }

    int last_error() const noexcept { return sqlite3_errcode(_db); }

protected:
    using select_callback_t = int (*)(void*, int, char**, char**);
    template <size_t N>
    using select_conditions_t = std::array<const char*, N>;

    sqlite3_db_t(const char* filename, int flags, const char* zVfs);
    virtual ~sqlite3_db_t();

    int open(const char* filename, int flags, const char* zVfs);

    template <typename... Args>
    int create_table(const char* table, Args&&... args);

    template <typename... Args>
    int insert(const char* table, Args&&... args);

    template <typename... Args>
    int insert_or_replace(const char* table, Args&&... args);

    int select_all(const char* table, select_callback_t cbk, void* cbk_arg);

    template <std::size_t N, typename... Args>
    int select_all(
        const char* table,
        select_callback_t cbk,
        void* cbk_arg,
        const select_conditions_t<N>& conditions,
        Args&&... args);

    int close();

    int exec(const char* sql, int (*callback)(void*, int, char**, char**), void*);

    bool _ok;

private:
    template <typename... Args>
    int insert_or_replace_impl(bool replace, const char* table, Args&&... args);

    sqlite3* _db;
};

template <typename... Args>
int sqlite3_db_t::create_table(const char* table, Args&&... args)
{
    const auto len = std::snprintf(nullptr, 0, create_table_stmt, table);
    auto create_str = std::make_unique<char[]>(len + 1);
    std::snprintf(create_str.get(), len + 1, create_table_stmt, table);

    const auto stmt = std::string{create_str.get()} + FLECS::stringify_delim(',', args...) + ");";

    return exec(stmt.c_str(), nullptr, nullptr);
}

template <typename... Args>
int sqlite3_db_t::insert(const char* table, Args&&... args)
{
    return insert_or_replace_impl(false, table, std::forward<Args>(args)...);
}

template <typename... Args>
int sqlite3_db_t::insert_or_replace(const char* table, Args&&... args)
{
    return insert_or_replace_impl(true, table, std::forward<Args>(args)...);
}

template <typename... Args>
int sqlite3_db_t::insert_or_replace_impl(bool replace, const char* table, Args&&... args)
{
    const auto len1 = std::snprintf(nullptr, 0, replace ? insert_or_replace_stmt : insert_stmt, table);
    auto insert_str = std::make_unique<char[]>(len1 + 1);
    std::snprintf(insert_str.get(), len1 + 1, replace ? insert_or_replace_stmt : insert_stmt, table);

    const auto stmt = std::string{insert_str.get()} + stringify_delim("\",\"", args...) + "\");";

    return exec(stmt.c_str(), nullptr, nullptr);
}

template <size_t N, typename... Args>
int sqlite3_db_t::select_all(
    const char* table, select_callback_t cbk, void* cbk_arg, const select_conditions_t<N>& conditions, Args&&... args)
{
    static_assert(N == sizeof...(args));

    const auto where = format_where(conditions.data(), args...);
    const auto len1 = std::snprintf(nullptr, 0, select_all_stmt, table);
    const auto len2 = where.length();

    auto select_str = std::make_unique<char[]>(len1 + len2 + 1);
    std::snprintf(select_str.get(), len1 + 1, select_all_stmt, table);
    std::snprintf(select_str.get() + len1, len2 + 1, "%s", where.c_str());

    return exec(select_str.get(), cbk, cbk_arg);
}

} // namespace FLECS

#endif // A46C0241_A55D_44A7_AAD0_70A31A41AFEA
