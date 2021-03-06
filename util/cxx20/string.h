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

#ifndef C54190C2_D535_4427_A5AA_82FC31D024D8
#define C54190C2_D535_4427_A5AA_82FC31D024D8

#include <string>
#include <string_view>

namespace FLECS {
namespace cxx20 {

template <typename CharT, typename Traits>
constexpr bool contains(std::basic_string_view<CharT, Traits> sv, std::basic_string_view<CharT, Traits> sv_cmp) noexcept
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
constexpr bool contains(const std::basic_string<CharT, Traits>& str, std::basic_string_view<CharT, Traits> sv) noexcept
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

template <typename CharT, typename Traits>
constexpr bool starts_with(
    std::basic_string_view<CharT, Traits> sv,
    std::basic_string_view<CharT, Traits> sv_cmp) noexcept
{
    return sv.substr(0, sv_cmp.size()) == sv_cmp;
}
template <typename CharT, typename Traits>
constexpr bool starts_with(std::basic_string_view<CharT, Traits> sv, CharT c) noexcept
{
    return !sv.empty() && Traits::eq(sv.front(), c);
}
template <typename CharT, typename Traits>
constexpr bool starts_with(std::basic_string_view<CharT, Traits> sv, const CharT* s)
{
    return starts_with(sv, std::basic_string_view<CharT, Traits>{s});
}

template <typename CharT, typename Traits>
constexpr bool starts_with(
    const std::basic_string<CharT, Traits>& str,
    std::basic_string_view<CharT, Traits> sv) noexcept
{
    return starts_with(std::basic_string_view<CharT, Traits>{str}, sv);
}
template <typename CharT, typename Traits>
constexpr bool starts_with(const std::basic_string<CharT, Traits>& str, CharT c) noexcept
{
    return starts_with(std::basic_string_view<CharT, Traits>{str}, c);
}
template <typename CharT, typename Traits>
constexpr bool starts_with(const std::basic_string<CharT, Traits>& str, const CharT* s)
{
    return starts_with(std::basic_string_view<CharT, Traits>{str}, s);
}

template <typename CharT>
constexpr bool starts_with(const CharT* str, std::basic_string_view<CharT> sv) noexcept
{
    return starts_with(std::basic_string_view<CharT>{str}, sv);
}

template <typename CharT>
constexpr bool starts_with(const CharT* str, CharT c) noexcept
{
    return starts_with(std::basic_string_view<CharT>{str}, c);
}

template <typename CharT>
constexpr bool starts_with(const CharT* str, const CharT* s) noexcept
{
    return starts_with(std::basic_string_view<CharT>{str}, s);
}

} // namespace cxx20
} // namespace FLECS

#endif // C54190C2_D535_4427_A5AA_82FC31D024D8
