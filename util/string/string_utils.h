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

#ifndef E3107886_7E4E_4556_9EDD_91B48A8DF4D9
#define E3107886_7E4E_4556_9EDD_91B48A8DF4D9

#include <algorithm>
#include <optional>
#include <sstream>
#include <string>
#include <string_view>
#include <type_traits>
#include <vector>

#include "core/global/types/type_traits.h"

namespace FLECS {

template <typename T>
std::enable_if_t<!std::is_convertible_v<T, std::string> && !std::is_same_v<std::decay_t<T>, bool>, std::string>
stringify_impl(T&& val)
{
    using std::to_string;
    return to_string(val);
}

template <typename T>
std::enable_if_t<!std::is_convertible_v<T, std::string> && std::is_same_v<std::decay_t<T>, bool>, std::string>
stringify_impl(T&& val)
{
    using std::operator""s;
    return val ? "true"s : "false"s;
}

template <typename T>
std::enable_if_t<std::is_convertible_v<T, std::string>, std::string> stringify_impl(T&& val)
{
    return static_cast<std::string>(val);
}

template <typename... Args>
std::string stringify(Args&&... args)
{
    auto str = std::string{};
    ((str += stringify_impl(args)), ...);
    return str;
}

template <typename T>
std::string stringify_delim_impl(std::string_view delim, T&& val)
{
    (void)delim; // suppress wrong unused warning
    if constexpr (is_std_container_v<T>) {
        auto str = std::string{};
        for (decltype(auto) it : val) {
            str += stringify_impl(it) + std::string{delim};
        }
        if (!str.empty()) {
            str.resize(str.size() - delim.size());
        }
        return str;
    } else {
        return stringify_impl(val);
    }
}

template <typename... Args>
std::string stringify_delim(std::string_view delim, Args&&... args)
{
    auto str = std::string{};
    ((str += stringify_delim_impl(delim, args) += delim), ...);
    str.resize(str.size() - delim.size());
    return str;
}

template <typename... Args>
std::string stringify_delim(char delim, Args&&... args)
{
    return stringify_delim(std::string_view{&delim, 1}, args...);
}

template <typename CharT, typename Traits>
auto split(const std::basic_string<CharT, Traits>& str, CharT delim)
{
    auto res = std::vector<std::basic_string<CharT, Traits>>{};
    auto iss = std::istringstream{str};
    auto item = std::basic_string<CharT, Traits>{};

    while (std::getline(iss, item, delim)) {
        res.emplace_back(item);
    }

    return res;
}

template <typename CharT, typename Traits>
inline auto split(const std::basic_string_view<CharT, Traits>& sv, CharT delim)
{
    return split(std::basic_string<CharT, Traits>{sv}, delim);
}

template <typename CharT>
inline auto split(const CharT* s, CharT delim)
{
    return split(std::basic_string<CharT>{s}, delim);
}

template <typename CharT, typename Traits>
inline auto ltrim(std::basic_string<CharT, Traits>& str) //
{
    str.erase(str.begin(), std::find_if(str.begin(), str.end(), [](CharT c) { return !std::isspace(c); }));
    return str;
}

template <typename CharT, typename Traits>
inline auto rtrim(std::basic_string<CharT, Traits>& str) //
{
    str.erase(std::find_if(str.rbegin(), str.rend(), [](CharT c) { return !std::isspace(c); }).base(), str.end());
    return str;
}

template <typename CharT, typename Traits>
inline auto trim(std::basic_string<CharT, Traits>& str) //
{
    ltrim(str);
    return rtrim(str);
}

} // namespace FLECS

#endif // E3107886_7E4E_4556_9EDD_91B48A8DF4D9
