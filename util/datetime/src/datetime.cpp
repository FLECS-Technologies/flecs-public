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

#include "datetime.h"

#include <type_traits>

namespace flecs {

using system_clock_t = std::chrono::system_clock;

static constexpr auto fmt_string_base = "%Y-%m-%dT%H:%M:%S";

static constexpr auto fmt_strings = std::array<const char*, 4>{
    ".%.9ldZ",
    ".%.6ldZ",
    ".%.3ldZ",
    "Z",
};

static constexpr auto fmt_divs = std::array<std::time_t, 4>{
    1'000'000'000,
    1'000'000,
    1'000,
    1,
};

static auto chrono_to_time_t(std::chrono::time_point<system_clock_t> tp, precision_e precision) //
    -> time_t
{
    const auto fmt_div = fmt_divs[static_cast<std::underlying_type_t<precision_e>>(precision)];
    const auto time =
        static_cast<std::time_t>(
            std::chrono::duration_cast<std::chrono::nanoseconds>(tp.time_since_epoch()).count()) /
        (fmt_divs[0] / fmt_div);
    return time;
}

auto time_to_iso(time_t time, precision_e precision) //
    -> std::string
{
    const auto fmt_div = fmt_divs[static_cast<std::underlying_type_t<precision_e>>(precision)];
    const auto time_s = time / fmt_div;

    auto time_utc = tm{};
    gmtime_r(&time_s, &time_utc);

    char strtime[32] = {};
    const auto pos = std::strftime(strtime, sizeof(strtime), fmt_string_base, &time_utc);

    const auto fmt_string =
        fmt_strings[static_cast<std::underlying_type_t<precision_e>>(precision)];
    std::snprintf(&strtime[pos], sizeof(strtime) - pos, fmt_string, time % fmt_div);

    return strtime;
}

auto time_to_iso(precision_e precision) //
    -> std::string
{
    return time_to_iso(system_clock_t::now(), precision);
}

auto time_to_iso(std::chrono::time_point<system_clock_t> tp, precision_e precision) //
    -> std::string
{
    const auto time = chrono_to_time_t(tp, precision);
    return time_to_iso(time, precision);
}

auto unix_time(precision_e precision) //
    -> std::string
{
    const auto time = chrono_to_time_t(system_clock_t::now(), precision);
    return std::to_string(time);
}

} // namespace flecs
