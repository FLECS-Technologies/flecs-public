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

#pragma once

#include <ctime>

#include "flecs/util/json/json.h"

namespace flecs::console {
class session_id_t
{
public:
    session_id_t();
    session_id_t(std::string id, std::time_t timestamp);

    auto id() const noexcept //
        -> const std::string&;

    auto timestamp() const noexcept //
        -> std::time_t;

private:
    friend auto from_json(const json_t& j, session_id_t& jwt) //
        -> void;
    friend auto to_json(json_t& j, const session_id_t& jwt) //
        -> void;

    std::string _id;
    time_t _timestamp;
};
} // namespace flecs::console
