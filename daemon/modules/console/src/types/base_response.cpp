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

#include "daemon/modules/console/types/base_response.h"

namespace flecs {
namespace console {

auto base_response_t::status_code() const noexcept //
    -> int
{
    return _status_code;
}

auto base_response_t::status_text() const noexcept //
    -> const std::string&
{
    return _status_text;
}

auto base_response_t::data() const noexcept //
    -> const json_t&
{
    return _data;
}

auto from_json(const json_t& j, base_response_t& response) //
    -> void
{
    j.at("status").get_to(response._status_code);
    j.at("statusText").get_to(response._status_text);
    j.at("data").get_to(response._data);
}

auto to_json(json_t& j, const base_response_t& response) //
    -> void
{
    j = json_t({
        {"status", response.status_code()},
        {"statusText", response.status_text()},
        {"data", response.data()},
    });
}

} // namespace console
} // namespace flecs
