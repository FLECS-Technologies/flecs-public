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

#include "flecs/modules/instances/types/instance_id.h"

#include <limits>

#include "flecs/util/random/random.h"
#include "flecs/util/string/format.h"

namespace flecs {
namespace instances {

id_t::id_t()
    : _id{rnd()}
{}

id_t::id_t(std::uint32_t id)
    : _id{id}
{}

id_t::id_t(std::string_view id)
    : _id{}
{
    auto end = static_cast<char*>(nullptr);
    const auto tmp = std::strtoul(id.data(), &end, 16);

    if (end != id.end() || (tmp > std::numeric_limits<decltype(_id)>::max())) {
        return;
    }

    _id = static_cast<decltype(_id)>(tmp);
}

auto id_t::get() const noexcept //
    -> std::uint32_t
{
    return _id;
}

auto id_t::hex() const //
    -> std::string
{
    return int_to_hex(get(), fmt::Lowercase, fmt::NoPrefix, fmt::LeadingZeroes);
}

auto id_t::regenerate() //
    -> void
{
    _id = rnd();
}

} // namespace instances
} // namespace flecs
