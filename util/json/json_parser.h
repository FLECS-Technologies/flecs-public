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

#ifndef A868B916_0CDC_4998_B52B_48094FB446C8
#define A868B916_0CDC_4998_B52B_48094FB446C8

#include <json/json.h>

#include <memory>
#include <optional>

namespace FLECS {

auto parse_json(const char* begin, const char* end) -> std::optional<Json::Value>;

inline auto parse_json(const std::string& str)
{
    return parse_json(str.c_str(), str.c_str() + str.length());
}

inline auto parse_json(std::string_view sv)
{
    return parse_json(sv.data(), sv.data() + sv.length());
}

inline auto parse_json(const char* str)
{
    return parse_json(str, str + std::strlen(str));
}

} // namespace FLECS

#endif // A868B916_0CDC_4998_B52B_48094FB446C8
