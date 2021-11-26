// Copyright 2021 FLECS Technologies GmbH
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

#ifndef FLECS_util_sqlite3_db_h
#define FLECS_util_sqlite3_db_h

#include "util/sqlite3_ext/sqlite3_statements.h"

#include "external/sqlite-3.36.0/sqlite3.h"

#include <array>
#include <cstdio>
#include <iostream>
#include <memory>
#include <string>

namespace FLECS {

template <typename T>
auto value_to_string(T&& value)
{
    using basic_type_t = std::remove_cv_t<
        std::remove_reference_t<T>>;

    auto str = std::string { "'" };
    if constexpr(std::is_same_v<int, basic_type_t> ||
        std::is_same_v<long, basic_type_t> ||
        std::is_same_v<long long, basic_type_t> ||
        std::is_same_v<unsigned, basic_type_t> ||
        std::is_same_v<unsigned long, basic_type_t> ||
        std::is_same_v<unsigned long long, basic_type_t> ||
        std::is_same_v<float, basic_type_t> ||
        std::is_same_v<double, basic_type_t> ||
        std::is_same_v<long double, basic_type_t>)
    {
        using std::to_string;
        str += to_string(value);
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
    const auto where = std::string {};
    return format_where_impl(where, condition, args...);
}

struct statement_format_t
{
    std::string prefix;
    std::string intermediate;
    std::string postfix;
};

template <typename Arg>
auto format_statement_impl(const statement_format_t& format, std::string statement, Arg&& arg)
{
    statement += value_to_string(arg);
    statement += format.postfix;
    return statement;
}

template <typename Arg, typename... Args>
auto format_statement_impl(const statement_format_t& format, std::string statement, Arg&& arg, Args&&... args)
{
    statement += value_to_string(arg);
    statement += format.intermediate;
    return format_statement_impl(format, statement, args...);
}

template <typename... Args>
auto format_statement(const statement_format_t& format, Args&&... args)
{
    std::string statement = format.prefix;
    return format_statement_impl(format, statement, args...);
}

template <typename... Args>
auto format_values(Args&&... args)
{
    auto format = statement_format_t {
        "VALUES (",
        ",",
        ");"
    };
    return format_statement(format, args...);
}

class sqlite3_db_t
{
public:
    using select_callback_t = int (*)(void*, int, char**, char**);
    template <size_t N>
    using select_conditions_t = std::array<const char*, N>;

    sqlite3_db_t(const char* filename, int flags, const char* zVfs);
    virtual ~sqlite3_db_t();

    template <typename... Args>
    int insert(const char* table, Args&&... args);

    template <typename... Args>
    int insert_or_replace(const char* table, Args&&... args);

    template <std::size_t N, typename... Args>
    int select_all(
        const char* table,
        select_callback_t cbk,
        void* cbk_arg,
        const select_conditions_t<N>& conditions,
        Args&&... args);

    int exec(const char* sql, int (*callback)(void*,int,char**,char**), void*);

    int close();

protected:
    bool _ok;

private:
    template <typename... Args>
    int insert_impl(bool replace, const char* table, Args&&... args);

    sqlite3* _db;
};

template <typename... Args>
int sqlite3_db_t::insert(const char* table, Args&&... args)
{
    return insert_impl(false, table, args...);
}

template <typename... Args>
int sqlite3_db_t::insert_or_replace(const char* table, Args&&... args)
{
    return insert_impl(true, table, args...);
}

template <typename... Args>
int sqlite3_db_t::insert_impl(bool replace, const char* table, Args&&... args)
{
    const auto values = format_values(args...);

    const auto len1 = std::snprintf(nullptr, 0,
        replace ? insert_or_replace_stmt : insert_stmt,
        table);
    const auto len2 = values.length();

    auto insert_str = std::make_unique<char[]>(len1 + len2 + 1);
    std::snprintf(insert_str.get(), len1 + 1,
        replace ? insert_or_replace_stmt : insert_stmt, table);
    std::snprintf(insert_str.get() + len1, len2 + 1, "%s", values.c_str());

    return exec(insert_str.get(), nullptr, nullptr);
}

template <size_t N, typename... Args>
int sqlite3_db_t::select_all(
    const char* table,
    select_callback_t cbk,
    void* cbk_arg,
    const select_conditions_t<N>& conditions,
    Args&&... args)
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

#endif // FLECS_util_sqlite3_db_h
