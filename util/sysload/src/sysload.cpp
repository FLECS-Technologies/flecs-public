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
#include <regex>

#include "util/datetime/datetime.h"

namespace FLECS {

sysload_t::sysload_t()
{
    // see man sysconf
    _core_count = sysconf(_SC_NPROCESSORS_ONLN);
    _clock_ticks = sysconf(_SC_CLK_TCK);

    _usage_total_old = std::vector<uint64_t>(_core_count + 1, 0);
    _idle_total_old = std::vector<uint64_t>(_core_count + 1, 0);

    // initialize old load values
    get_cpu_load();

    // initialize flunder client
    _flunder_client = flunder_client_t{};
    _flunder_client.connect();
}

/** @todo FLX-241*/
auto sysload_t::check_connection() //
    -> void
{
    if (!_flunder_client.is_connected()) {
        _flunder_client.connect();
    }
}

auto sysload_t::update_load() //
    -> void
{
    _cpu_load = get_cpu_load();
    _cpu_clock = get_cpu_clock();
    _ram_load = get_ram_load();
    _uptime = get_uptime();
}

auto sysload_t::publish_load() //
    -> void
{
    auto base_topic = std::string("/flecs/system/");

    // Publish CPU load info
    for (int i = 0; i <= _core_count; i++) {
        auto message = _cpu_load[i];

        if (i == 0) {
            _flunder_client.publish(base_topic + "cpu/load", message);
        } else {
            auto core_num = std::to_string(i - 1);
            _flunder_client.publish(base_topic + "cpu/" + core_num + "/load", message);
        }
    }
    // Publish CPU clock info (in MHz)
    for (int i = 0; i < _core_count; i++) {
        auto message = _cpu_clock[i];
        auto core_num = std::to_string(i);

        _flunder_client.publish(base_topic + "cpu/" + core_num + "/clock", message);
    }

    // Publish RAM info (in GiB)
    auto message_avail = (double)_ram_load[0] / (1 << 30);
    auto message_total = (double)_ram_load[1] / (1 << 30);

    _flunder_client.publish(base_topic + "mem/available", message_avail);
    _flunder_client.publish(base_topic + "mem/total", message_total);

    // Publish uptime
    auto uptime_seconds = static_cast<unsigned long int>(_uptime.count());

    const auto len = std::snprintf(
        nullptr,
        0,
        "%lu days %lu hours %lu minutes %lu seconds",
        uptime_seconds / 86400,
        (uptime_seconds % 86400) / 3600,
        ((uptime_seconds % 86400) % 3600) / 60,
        ((uptime_seconds % 86400) % 3600) % 60);
    auto message = std::string(len, '\0');
    std::snprintf(
        message.data(),
        len + 1,
        "%lu days %lu hours %lu minutes %lu seconds",
        uptime_seconds / 86400,
        (uptime_seconds % 86400) / 3600,
        ((uptime_seconds % 86400) % 3600) / 60,
        ((uptime_seconds % 86400) % 3600) % 60);

    _flunder_client.publish(base_topic + "uptime", message);
}

auto sysload_t::get_cpu_load() //
    -> std::vector<float>
{
    // open input stream
    std::ifstream cpu_read;
    auto constexpr path = "/proc/stat";
    cpu_read.open(path);

    auto cpu_core_loads = std::vector<float>(_core_count + 1); // total cpu load at [0]

    // read lines until core_count is reached
    std::string cpunum;
    uint64_t statline[10];
    // each line contains: time spent in mode N, measured in USER_HZ (typically 1/100s)
    // 0=user, 1=nice, 2=system, 3=idle, 4=iowait, 5=irq, 6=softirq, 7=steal, 8=guest,
    // 9=guest_nice
    for (int i = 0; i <= _core_count; i++) {
        if (not cpu_read.good() or cpu_read.peek() != 'c') {
            std::cerr << "Read error in " << path << std::endl;
        } else {
            auto time_total = 0L;
            cpu_read >> cpunum;
            for (int j = 0; j < 10; j++) {
                cpu_read >> statline[j];
                time_total += statline[j];
            }

            // steal, guest and guest_nice included both for total and usage atm
            //(btop includes steal, but excludes guest and guest_nice?)
            uint64_t idle_total = statline[3] + statline[4]; // idle and iowait
            uint64_t usage_total = time_total - idle_total;
            auto usage_diff = usage_total - _usage_total_old[i];
            auto idle_diff = idle_total - _idle_total_old[i];
            auto cpu_util = ((float)usage_diff) / (usage_diff + idle_diff);

            cpu_core_loads[i] = cpu_util;

            _usage_total_old[i] = usage_total;
            _idle_total_old[i] = idle_total;

            cpu_read.get(); // read and discard newline char
        }
    }
    return cpu_core_loads;
}

auto sysload_t::get_cpu_clock() //
    -> std::vector<float>
{
    auto cpu_mhz = std::vector<float>(_core_count, 0);
    auto constexpr path = "/proc/cpuinfo";
    std::ifstream cpu_read(path);

    int i = 0;
    while (cpu_read.good() and i < _core_count) {
        std::string line_in;
        std::getline(cpu_read, line_in);

        std::smatch line_out;
        auto regex = std::regex(R"(cpu\sMHz\s+:\s*(\d+.\d+))");
        auto does_match = std::regex_match(line_in, line_out, regex);

        if (does_match) {
            try {
                cpu_mhz[i] = std::stof(line_out[1].str());
            } catch (const std::exception& e) {
                std::cerr << e.what() << std::endl;
            }
            i++;
        }
    }
    return cpu_mhz;
}

auto sysload_t::get_ram_load() //
    -> std::vector<uint64_t>
{
    std::ifstream mem_read;
    auto constexpr path = "/proc/meminfo";
    mem_read.open(path);

    auto result = std::vector<uint64_t>{0, 0}; // available, total in B

    std::string name;
    std::string unit;
    uint64_t value;

    for (int i = 0; mem_read.good(); i++) {
        mem_read >> name >> value >> unit;
        if (unit == "kB") {
            value *= 1024;
        } else {
            // TODO scale based on unit
        }

        if (name == "MemTotal:") {
            result[1] = value;
        } else if (name == "MemAvailable:") {
            result[0] = value;
        }
    }
    return result;
}

auto sysload_t::get_uptime() //
    -> std::chrono::seconds
{
    std::ifstream uptime_read;
    auto constexpr path = "/proc/uptime";
    uptime_read.open(path);

    auto uptime_seconds = 0.0f;
    uptime_read >> uptime_seconds;
    uptime_seconds = floor(uptime_seconds);

    return std::chrono::seconds{(int)uptime_seconds};
}

} // namespace FLECS
