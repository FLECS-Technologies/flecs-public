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

#include "sysload.h"

#include <unistd.h>

#include <fstream>
#include <iostream>

#include "util/datetime/datetime.h"

namespace FLECS {

sysload_t::sysload_t()
    : _core_count{}
    , _clock_ticks{}
{
    // see man sysconf
    _core_count = sysconf(_SC_NPROCESSORS_ONLN);
    _clock_ticks = sysconf(_SC_CLK_TCK);

    _usage_total_old = std::vector<unsigned long>(_core_count + 1, 0);
    _idle_total_old = std::vector<unsigned long>(_core_count + 1, 0);

    // initialize old load values
    get_cpu_load();

    // initialize flunder client
    flunder_client.connect();
}

void sysload_t::update_load()
{
    _cpu_load = get_cpu_load();
    _cpu_clock = get_cpu_clock();
    _ram_load = get_ram_load();
    _uptime = get_uptime();
}

void sysload_t::publish_load()
{
    std::string base_topic{"/flecs/system/"};

    // Publish CPU load info
    // Publish CPU clock info (in MHz)
    // Publish RAM info (in GiB)
    // Publish uptime
}

auto sysload_t::get_cpu_load() //
    -> std::vector<float>
{
    std::vector<float> cpu_core_loads(_core_count + 1, -1);
    return cpu_core_loads;
}

auto sysload_t::get_cpu_clock() //
    -> std::vector<float>
{
    auto cpu_mhz = std::vector<float>(_core_count, 0);
    return cpu_mhz;
}

auto sysload_t::get_ram_load() //
    -> std::vector<uint64_t>
{
    std::vector<uint64_t> result{0, 0}; // available, total in B
    return result;
}

auto sysload_t::get_uptime() //
    -> std::chrono::duration<float>
{
    float uptime_seconds = 0;
    return std::chrono::duration<float>{uptime_seconds};
}

} // namespace FLECS
