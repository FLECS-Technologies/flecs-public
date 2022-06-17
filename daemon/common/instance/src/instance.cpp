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

#include "instance.h"

#include <cstdio>
#include <random>

namespace FLECS {

std::string generate_instance_id()
{
    auto res = std::string(8, '\0');

    auto seed = std::random_device{};
    auto generator = std::mt19937{seed()};
    auto distribution = std::uniform_int_distribution{
        std::numeric_limits<std::uint32_t>::min(),
        std::numeric_limits<std::uint32_t>::max()};

    auto id = distribution(generator);
    std::snprintf(res.data(), res.length() + 1, "%.8x", id);

    return res;
}

instance_t::instance_t(
    std::string app,
    std::string version,
    std::string instance_name,
    instance_status_e status,
    instance_status_e desired)
    : instance_t{generate_instance_id(), app, version, instance_name, status, desired}
{}

instance_t::instance_t(
    std::string id,
    std::string app,
    std::string version,
    std::string instance_name,
    instance_status_e status,
    instance_status_e desired)
    : _id{id}
    , _app{app}
    , _version{version}
    , _instance_name{instance_name}
    , _status{status}
    , _desired{desired}
{}

auto instance_t::regenerate_id() //
    -> void
{
    _id = generate_instance_id();
}

} // namespace FLECS
