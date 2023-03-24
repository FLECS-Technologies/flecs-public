// Copyright 2021-2023 FLECS Technologies GmbH
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

#pragma once

#ifdef __cplusplus

#include <cstdlib>
#include <string>
#include <string_view>
#include <variant>

namespace FLECS {

class flunder_variable_t
{
public:
    FLECS_EXPORT flunder_variable_t();

    FLECS_EXPORT flunder_variable_t(
        std::string topic, std::string value, std::string encoding, std::string timestamp);
    FLECS_EXPORT flunder_variable_t(
        const char* topic, const char* value, const char* encoding, const char* timestamp);

    FLECS_EXPORT auto topic() const noexcept //
        -> std::string_view;
    FLECS_EXPORT auto value() const noexcept //
        -> std::string_view;
    FLECS_EXPORT auto len() const noexcept //
        -> std::size_t;
    FLECS_EXPORT auto encoding() const noexcept //
        -> std::string_view;
    FLECS_EXPORT auto timestamp() const noexcept //
        -> std::string_view;

    FLECS_EXPORT auto own() //
        -> void;
    FLECS_EXPORT auto is_owned() const noexcept //
        -> bool;

private:
    std::variant<std::string_view, std::string> _topic;
    std::variant<std::string_view, std::string> _value;
    std::variant<std::string_view, std::string> _encoding;
    std::variant<std::string_view, std::string> _timestamp;
};

} // namespace FLECS

#else // __cplusplus

#include <stdbool.h>
#include <stdlib.h>

typedef struct flunder_variable_t flunder_variable_t;

#endif //__cplusplus

#ifdef __cplusplus
extern "C" {

using flunder_variable_t = FLECS::flunder_variable_t;
#endif //__cplusplus

FLECS_EXPORT flunder_variable_t* flunder_variable_new(
    const char* key, const char* value, const char* encoding, const char* timestamp);

FLECS_EXPORT flunder_variable_t* flunder_variable_clone(const flunder_variable_t* other);

FLECS_EXPORT flunder_variable_t* flunder_variable_move(flunder_variable_t* other);

FLECS_EXPORT const char* flunder_variable_topic(const flunder_variable_t* var);
FLECS_EXPORT const char* flunder_variable_value(const flunder_variable_t* var);
FLECS_EXPORT size_t flunder_variable_len(const flunder_variable_t* var);
FLECS_EXPORT const char* flunder_variable_encoding(const flunder_variable_t* var);
FLECS_EXPORT const char* flunder_variable_timestamp(const flunder_variable_t* var);

FLECS_EXPORT void flunder_variable_destroy(flunder_variable_t* var);
FLECS_EXPORT void flunder_variable_list_destroy(flunder_variable_t* vars, size_t n);
FLECS_EXPORT const flunder_variable_t* flunder_variable_next(const flunder_variable_t* var);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
