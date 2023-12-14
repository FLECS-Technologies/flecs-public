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

namespace flecs {
namespace instances {

class id_t
{
public:
    id_t();

    id_t(std::uint32_t id);

    id_t(std::string_view id);

    auto get() const noexcept //
        -> std::uint32_t;

    auto hex() const //
        -> std::string;

    auto regenerate() //
        -> void;

private:
    friend auto operator<=>(const id_t&, const id_t&) = default;

    std::uint32_t _id;
};

} // namespace instances
} // namespace flecs
