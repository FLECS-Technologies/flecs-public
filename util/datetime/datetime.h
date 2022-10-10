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

#ifndef F945857A_262A_4302_A4C9_B209858CFA98
#define F945857A_262A_4302_A4C9_B209858CFA98

#include <array>
#include <chrono>
#include <string>
#include <utility>

namespace FLECS {

using system_clock_t = std::chrono::system_clock;

enum class precision_e : std::size_t {
    nanoseconds,  ///!< nanosecond precision
    microseconds, ///!< microsecond precision
    milliseconds, ///!< millisecond precision
    seconds,      ///!< second precision
};

/**! @brief Converts a given time point in localtime to an ISO 8601 string in UTC
 *
 * @param[in] time localtime point obtained from any realtime-clock
 * @param[in] precision precision between seconds and nanoseconds @sa precision_e
 *
 * @return std::string containing ISO 8601 time
 */
std::string time_to_iso(std::time_t time, precision_e precision = precision_e::milliseconds);

/**! @brief Returns the current time as an ISO 8601 string in UTC
 *
 * Time is obtained from @sa system_clock_t (a.k.a. std::chrono::system_clock)
 *
 * @param[in] precision precision between seconds and nanoseconds @sa precision_e
 */
std::string time_to_iso(precision_e precision = precision_e::milliseconds);

/**! @brief Converts a given chrono time point in localtime to an ISO 8601 string in UTC
 *
 * @param[in] tp time point obtained from @sa system_clock_t (a.k.a. std::chrono::system_clock)
 * @param[in] precision precision between seconds and nanoseconds @sa precision_e
 */
std::string time_to_iso(std::chrono::time_point<system_clock_t> tp, precision_e precision = precision_e::milliseconds);

} // namespace FLECS

#endif // F945857A_262A_4302_A4C9_B209858CFA98
