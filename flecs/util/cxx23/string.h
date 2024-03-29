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

#include <string>
#include <string_view>

namespace flecs {
namespace cxx23 {

template <typename CharT, typename Traits>
constexpr bool contains(
    std::basic_string_view<CharT, Traits> sv, std::basic_string_view<CharT, Traits> sv_cmp) noexcept
{
    return sv.find(sv_cmp) != std::basic_string_view<CharT, Traits>::npos;
}
template <typename CharT, typename Traits>
constexpr bool contains(std::basic_string_view<CharT, Traits> sv, CharT c) noexcept
{
    return sv.find(c) != std::basic_string_view<CharT, Traits>::npos;
}
template <typename CharT, typename Traits>
constexpr bool contains(std::basic_string_view<CharT, Traits> sv, const CharT* s)
{
    return contains(sv, std::basic_string_view<CharT, Traits>{s});
}

template <typename CharT, typename Traits>
constexpr bool contains(
    const std::basic_string<CharT, Traits>& str, std::basic_string_view<CharT, Traits> sv) noexcept
{
    return contains(std::basic_string_view<CharT, Traits>{str}, sv);
}
template <typename CharT, typename Traits>
constexpr bool contains(const std::basic_string<CharT, Traits>& str, CharT c) noexcept
{
    return contains(std::basic_string_view<CharT, Traits>{str}, c);
}
template <typename CharT, typename Traits>
constexpr bool contains(const std::basic_string<CharT, Traits>& str, const CharT* s)
{
    return contains(std::basic_string_view<CharT, Traits>{str}, s);
}

template <typename CharT>
constexpr bool contains(const CharT* str, std::basic_string_view<CharT> sv)
{
    return contains(std::basic_string_view<CharT>{str}, sv);
}
template <typename CharT>
constexpr bool contains(const CharT* str, CharT c)
{
    return contains(std::basic_string_view<CharT>{str}, c);
}
template <typename CharT>
constexpr bool contains(const CharT* str, const CharT* s)
{
    return contains(std::basic_string_view<CharT>{str}, s);
}

} // namespace cxx23
} // namespace flecs
