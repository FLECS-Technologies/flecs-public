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

#include <cstdint>
#include <string>
#include <string_view>

#include "util/json/json.h"

namespace FLECS {

class instance_id_t
{
public:
    instance_id_t();

    instance_id_t(std::uint32_t id);

    instance_id_t(std::string_view id);

    auto get() const noexcept //
        -> std::uint32_t;

    auto hex() const //
        -> std::string;

    auto regenerate() //
        -> void;

private:
    friend auto operator<=>(const instance_id_t&, const instance_id_t&) = default;

    std::uint32_t _id;
};

} // namespace FLECS
