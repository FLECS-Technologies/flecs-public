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

#ifndef FLECS_util_string_utils_h
#define FLECS_util_string_utils_h

#include <optional>
#include <sstream>
#include <string>
#include <string_view>
#include <type_traits>
#include <vector>

namespace FLECS {

template <typename T>
std::enable_if_t<!std::is_convertible_v<T, std::string>, std::string> stringify_impl(T&& val)
{
    using std::to_string;
    return to_string(val);
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

template <typename... Args>
std::string stringify_delim(char delim, Args&&... args)
{
    (void)delim;
    auto str = std::string{};
    ((str += stringify_impl(args) += delim), ...);
    str.pop_back();
    return str;
}

template <typename... Args>
std::string stringify_delim(std::string delim, Args&&... args)
{
    auto str = std::string{};
    ((str += stringify_impl(args) += delim), ...);
    str.resize(str.size() - delim.size());
    return str;
}

template <typename CharT, typename Traits>
auto split(const std::basic_string<CharT, Traits>& str, CharT delim)
{
    auto res = std::vector<std::basic_string<CharT, Traits>>{};
    auto iss = std::istringstream{str};
    auto item = std::basic_string<CharT, Traits>{};

    while (std::getline(iss, item, delim))
    {
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

} // namespace FLECS

#endif // FLECS_util_string_utils_h
