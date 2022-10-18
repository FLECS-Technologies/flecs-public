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

#include "flunder_variable.h"

#include <cstring>
#include <memory>

namespace FLECS {

/** @todo */
template <class... Ts>
struct overload : Ts...
{
    using Ts::operator()...;
};
template <class... Ts>
overload(Ts...) -> overload<Ts...>;

FLECS_EXPORT flunder_variable_t::flunder_variable_t()
    : _topic{}
    , _value{}
    , _encoding{}
    , _timestamp{}
{}

FLECS_EXPORT flunder_variable_t::flunder_variable_t(
    std::string key, std::string value, std::string encoding, std::string timestamp)
    : _topic(std::move(key))
    , _value(std::move(value))
    , _encoding(std::move(encoding))
    , _timestamp(std::move(timestamp))
{}

FLECS_EXPORT flunder_variable_t::flunder_variable_t(
    const char* key, const char* value, const char* encoding, const char* timestamp)
    : _topic(std::string_view{key})
    , _value(std::string_view{value})
    , _encoding(std::string_view{encoding})
    , _timestamp(std::string_view{timestamp})
{}

template <typename... Ts>
auto as_string_view(std::variant<Ts...> var) //
    -> std::string_view
{
    return std::visit(
        overload{
            [&](const std::string& str) -> std::string_view { return str; },
            [&](const std::string_view sv) -> std::string_view { return sv; }},
        var);
}

FLECS_EXPORT auto flunder_variable_t::topic() const noexcept //
    -> std::string_view
{
    return as_string_view(_topic);
}

FLECS_EXPORT auto flunder_variable_t::value() const noexcept //
    -> std::string_view
{
    return as_string_view(_topic);
}

FLECS_EXPORT auto flunder_variable_t::len() const noexcept //
    -> std::size_t
{
    return value().size();
}

FLECS_EXPORT auto flunder_variable_t::encoding() const noexcept //
    -> std::string_view
{
    return as_string_view(_encoding);
}

FLECS_EXPORT auto flunder_variable_t::timestamp() const noexcept //
    -> std::string_view
{
    return as_string_view(_timestamp);
}

FLECS_EXPORT auto flunder_variable_t::own() //
    -> void
{
    if (!is_owned())
    {
        _topic = std::string{std::get<std::string_view>(_topic)};
        _value = std::string{std::get<std::string_view>(_value)};
        _encoding = std::string{std::get<std::string_view>(_encoding)};
        _timestamp = std::string{std::get<std::string_view>(_timestamp)};
    }
}

FLECS_EXPORT auto flunder_variable_t::is_owned() const noexcept //
    -> bool
{
    return std::holds_alternative<std::string>(_topic);
}

} // namespace FLECS

extern "C" {

FLECS_EXPORT flunder_variable_t* flunder_variable_new(
    const char* key, const char* value, const char* encoding, const char* timestamp)
{
    return new flunder_variable_t{key, value, encoding, timestamp};
}

FLECS_EXPORT flunder_variable_t* flunder_variable_clone(const flunder_variable_t* other)
{
    return new flunder_variable_t{*other};
}

FLECS_EXPORT flunder_variable_t* flunder_variable_move(flunder_variable_t* other)
{
    return new flunder_variable_t{std::move(*other)};
}

FLECS_EXPORT const char* flunder_variable_topic(const flunder_variable_t* var)
{
    return var->topic().data();
}

FLECS_EXPORT const char* flunder_variable_value(const flunder_variable_t* var)
{
    return var->value().data();
}

FLECS_EXPORT size_t flunder_variable_len(const flunder_variable_t* var)
{
    return var->len();
}

FLECS_EXPORT const char* flunder_variable_encoding(const flunder_variable_t* var)
{
    return var->encoding().data();
}

FLECS_EXPORT const char* flunder_variable_timestamp(const flunder_variable_t* var)
{
    return var->timestamp().data();
}

FLECS_EXPORT void flunder_variable_destroy(flunder_variable_t* var)
{
    delete var;
}

} // extern "C"
