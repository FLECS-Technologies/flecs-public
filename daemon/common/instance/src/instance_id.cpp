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

#include "instance_id.h"

#include <limits>

#include "util/random/random.h"
#include "util/string/format.h"

namespace FLECS {

instance_id_t::instance_id_t()
    : _id{rnd()}
{}

instance_id_t::instance_id_t(std::uint32_t id)
    : _id{id}
{}

instance_id_t::instance_id_t(std::string_view id)
    : _id{}
{
    auto end = static_cast<char*>(nullptr);
    const auto tmp = std::strtoul(id.data(), &end, 16);

    if (end != id.end() || (tmp > std::numeric_limits<decltype(_id)>::max())) {
        return;
    }

    _id = static_cast<decltype(_id)>(tmp);
}

auto instance_id_t::get() const noexcept //
    -> std::uint32_t
{
    return _id;
}

auto instance_id_t::hex() const //
    -> std::string
{
    return int_to_hex(get(), fmt::Lowercase, fmt::NoPrefix, fmt::LeadingZeroes);
}

auto instance_id_t::regenerate() //
    -> void
{
    _id = rnd();
}

auto to_json(json_t& j, const instance_id_t& instance_id) //
    -> void
{
    j = json_t({{"instanceId", instance_id.hex()}});
}

auto from_json(const json_t& j, instance_id_t& instance_id) //
    -> void
{
    instance_id = instance_id_t{};
    if (j.contains("instanceId")) {
        auto id = std::stoul(j["instanceId"].get_ref<const std::string&>(), nullptr, 16);
        instance_id = instance_id_t{static_cast<std::uint32_t>(id)};
    }
}

auto operator<(const instance_id_t& lhs, const instance_id_t& rhs) //
    -> bool
{
    return lhs.get() < rhs.get();
}

auto operator<=(const instance_id_t& lhs, const instance_id_t& rhs) //
    -> bool
{
    return !(lhs > rhs);
}

auto operator>(const instance_id_t& lhs, const instance_id_t& rhs) //
    -> bool
{
    return rhs < lhs;
}

auto operator>=(const instance_id_t& lhs, const instance_id_t& rhs) //
    -> bool
{
    return !(lhs < rhs);
}

auto operator==(const instance_id_t& lhs, const instance_id_t& rhs) //
    -> bool
{
    return lhs.get() == rhs.get();
}

auto operator!=(const instance_id_t& lhs, const instance_id_t& rhs) //
    -> bool
{
    return !(lhs == rhs);
}

} // namespace FLECS
