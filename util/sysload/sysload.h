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

#include <cinttypes>
#include <fstream>
#include <string>

#include "flunder/flunder_client.h"
#include "util/fs/fs.h"
#include "util/json/json.h"

namespace FLECS {

class sysload_t
{
public:
    sysload_t();

    auto update_load() //
        -> void;
    auto publish_load() //
        -> void;
    auto check_connection() //
        -> void;

private:
    auto get_cpu_load() //
        -> std::vector<float>;
    auto get_cpu_clock() //
        -> std::vector<float>;

    auto get_ram_load() //
        -> std::vector<uint64_t>;

    auto get_uptime() //
        -> std::chrono::seconds;

    // Processes
    // get number of running processes
    auto get_process_count() //
        -> int;
    // get CPU load for flecs-core, flecs-webapp and each installed app
    auto get_app_utilization() //
        -> std::vector<float>;

    // CPU
    int _core_count;
    std::vector<float> _cpu_load;
    std::vector<uint64_t> _usage_total_old;
    std::vector<uint64_t> _idle_total_old;
    std::vector<float> _cpu_clock;

    // RAM
    // get RAM usage (avail, total). 32 bits may not be enough (256GiB needs 40 bits)
    std::vector<uint64_t> _ram_load;

    // Uptime
    // get system uptime (in seconds)
    std::chrono::seconds _uptime;

    // Misc
    int _clock_ticks; // number of clock ticks per second
    flunder_client_t _flunder_client;
};

} // namespace FLECS
