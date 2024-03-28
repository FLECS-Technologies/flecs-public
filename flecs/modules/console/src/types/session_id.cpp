// Copyright 2021-2024 FLECS Technologies GmbH
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

#include "flecs/modules/console/types/session_id.h"

namespace flecs::console {
session_id_t::session_id_t()
    : _id{}
    , _timestamp{}
{}

session_id_t::session_id_t(std::string id, std::time_t timestamp)
    : _id(std::move(id))
    , _timestamp(timestamp)
{}

auto session_id_t::id() const noexcept //
    -> const std::string&
{
    return _id;
}

auto session_id_t::timestamp() const noexcept //
    -> time_t
{
    return _timestamp;
}

auto from_json(
    const json_t& j,
    session_id_t& session_id) //
    -> void
{
    j.at("id").get_to(session_id._id);
    j.at("timestamp").get_to(session_id._timestamp);
}

auto to_json(
    json_t& j,
    const session_id_t& session_id) //
    -> void
{
    j = json_t({
        {"id", session_id.id()},
        {"timestamp", session_id.timestamp()},
    });
}
} // namespace flecs::console