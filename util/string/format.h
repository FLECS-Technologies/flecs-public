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

#include <iomanip>
#include <sstream>
#include <string>

#include "core/global/types/type_traits.h"

namespace FLECS {
namespace impl {

struct bin_traits_t
{
    template <typename T>
    static constexpr auto size(T)
    {
        return 8 * sizeof(T);
    }
    static constexpr const char* prefix[] = {"0b", "0B"};
};
inline std::ostream& operator<<(std::ostream& os, const bin_traits_t&)
{
    return os;
}
template <typename T>
struct oct_traits_t
{
    static constexpr auto size() { return (8 * sizeof(T) + 2) / 3; }
    static constexpr const char* prefix[] = {"0", "0"};
};
template <typename T>
inline std::ostream& operator<<(std::ostream& os, const oct_traits_t<T>&)
{
    os << std::oct;
    return os;
}
template <typename T>
struct dec_traits_t
{
    static constexpr auto size() { return (8 * sizeof(T) + 2) / 3; }
    static constexpr const char* prefix[] = {"", ""};
};
template <typename T>
inline std::ostream& operator<<(std::ostream& os, const dec_traits_t<T>&)
{
    os << std::dec;
    return os;
}
template <typename T>
struct hex_traits_t
{
    static constexpr auto size() { return (8 * sizeof(T)) / 4; }
    static constexpr const char* prefix[] = {"0x", "0X"};
};
template <typename T>
inline std::ostream& operator<<(std::ostream& os, const hex_traits_t<T>&)
{
    os << std::hex;
    return os;
}

template <typename Traits>
struct fmt_ctx_t
{
    bool _uppercase = false;
    bool _prefix = false;
    bool _leading_zeroes = true;
};
template <typename Traits>
inline std::ostream& operator<<(std::ostream& os, const fmt_ctx_t<Traits>& fmt_ctx)
{
    os << Traits{};
    os << (fmt_ctx._uppercase ? std::uppercase : std::nouppercase);
    os << (fmt_ctx._prefix ? Traits::prefix[fmt_ctx._uppercase] : "");
    os << std::setw(fmt_ctx._leading_zeroes ? Traits::size() : 0);
    os << std::setfill('0');
    return os;
}
} // namespace impl

namespace fmt {
enum case_e {
    Lowercase = 0,
    Uppercase = 1,
};
enum prefix_e {
    NoPrefix = 0,
    Prefix = 1,
};
enum leading_zeroes_e {
    NoLeadingZeroes = 0,
    LeadingZeroes = 1,
};
} // namespace fmt

template <typename T>
inline auto int_to_hex(
    T val, fmt::case_e casing, fmt::prefix_e prefix, fmt::leading_zeroes_e leading_zeroes) //
    -> std::enable_if_t<std::is_integral_v<T>, std::string>
{
    const auto fmt = impl::fmt_ctx_t<impl::hex_traits_t<T>>{
        ._uppercase = static_cast<bool>(casing),
        ._prefix = static_cast<bool>(prefix),
        ._leading_zeroes = static_cast<bool>(leading_zeroes),
    };

    auto ss = std::stringstream{};
    ss << fmt << val;
    return ss.str();
}

} // namespace FLECS
