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

#ifndef B2BC57AF_1CAA_4C27_9C4F_940B766B7E3C
#define B2BC57AF_1CAA_4C27_9C4F_940B766B7E3C

#include <array>
#include <list>
#include <map>
#include <set>
#include <string>
#include <string_view>
#include <vector>

#include "core/global/types/type_traits.h"
#include "core/global/types/types.h"

namespace FLECS {

constexpr cstr_t type_name(void_t) noexcept
{
    return "void";
}

constexpr cstr_t type_name(bool_t) noexcept
{
    return "bool";
}
constexpr cstr_t type_name(int8_t) noexcept
{
    return "sint8";
}
constexpr cstr_t type_name(uint8_t) noexcept
{
    return "uint8";
}
constexpr cstr_t type_name(int16_t) noexcept
{
    return "sint16";
}
constexpr cstr_t type_name(uint16_t) noexcept
{
    return "uint16";
}
constexpr cstr_t type_name(int32_t) noexcept
{
    return "sint32";
}
constexpr cstr_t type_name(uint32_t) noexcept
{
    return "uint32";
}
constexpr cstr_t type_name(int64_t) noexcept
{
    return "sint64";
}
constexpr cstr_t type_name(uint64_t) noexcept
{
    return "uint64";
}
constexpr cstr_t type_name(fp32_t) noexcept
{
    return "fp32";
}
constexpr cstr_t type_name(fp64_t) noexcept
{
    return "fp64";
}

constexpr cstr_t type_name(void_t*) noexcept
{
    return "void_ptr";
}
constexpr cstr_t type_name(bool_t*) noexcept
{
    return "bool_ptr";
}
constexpr cstr_t type_name(int8_t*) noexcept
{
    return "sint8_ptr";
}
constexpr cstr_t type_name(uint8_t*) noexcept
{
    return "uint8_ptr";
}
constexpr cstr_t type_name(int16_t*) noexcept
{
    return "sint16_ptr";
}
constexpr cstr_t type_name(uint16_t*) noexcept
{
    return "uint16_ptr";
}
constexpr cstr_t type_name(int32_t*) noexcept
{
    return "sint32_ptr";
}
constexpr cstr_t type_name(uint32_t*) noexcept
{
    return "uint32_ptr";
}
constexpr cstr_t type_name(int64_t*) noexcept
{
    return "sint64_ptr";
}
constexpr cstr_t type_name(uint64_t*) noexcept
{
    return "uint64_ptr";
}
constexpr cstr_t type_name(fp32_t*) noexcept
{
    return "fp32_ptr";
}
constexpr cstr_t type_name(fp64_t*) noexcept
{
    return "fp64_ptr";
}

constexpr cstr_t type_name(const void_t*) noexcept
{
    return "void_ptr";
}
constexpr cstr_t type_name(const bool_t*) noexcept
{
    return "bool_ptr";
}
constexpr cstr_t type_name(const int8_t*) noexcept
{
    return "sint8_ptr";
}
constexpr cstr_t type_name(const uint8_t*) noexcept
{
    return "uint8_ptr";
}
constexpr cstr_t type_name(const int16_t*) noexcept
{
    return "sint16_ptr";
}
constexpr cstr_t type_name(const uint16_t*) noexcept
{
    return "uint16_ptr";
}
constexpr cstr_t type_name(const int32_t*) noexcept
{
    return "sint32_ptr";
}
constexpr cstr_t type_name(const uint32_t*) noexcept
{
    return "uint32_ptr";
}
constexpr cstr_t type_name(const int64_t*) noexcept
{
    return "sint64_ptr";
}
constexpr cstr_t type_name(const uint64_t*) noexcept
{
    return "uint64_ptr";
}
constexpr cstr_t type_name(const fp32_t*) noexcept
{
    return "fp32_ptr";
}
constexpr cstr_t type_name(const fp64_t*) noexcept
{
    return "fp64_ptr";
}

constexpr cstr_t type_name(cstr_t) noexcept
{
    return "string";
}
constexpr cstr_t type_name(const std::string&) noexcept
{
    return "string";
}
constexpr cstr_t type_name(const std::string_view&) noexcept
{
    return "string";
}
template <size_t N>
constexpr cstr_t type_name(char_t (&)[N]) noexcept
{
    return "string";
}

constexpr cstr_t type_name(wcstr_t) noexcept
{
    return "wstring";
}
constexpr cstr_t type_name(const std::wstring&) noexcept
{
    return "wstring";
}
constexpr cstr_t type_name(const std::wstring_view&) noexcept
{
    return "wstring";
}
template <size_t N>
constexpr cstr_t type_name(wchar_t (&)[N]) noexcept
{
    return "string";
}

constexpr cstr_t type_name(u16cstr_t) noexcept
{
    return "u16string";
}

constexpr cstr_t type_name(u32cstr_t) noexcept
{
    return "u32string";
}

template <typename T>
constexpr cstr_t type_name(const std::list<T>&) noexcept
{
    return "list";
}

template <typename T, typename U>
constexpr cstr_t type_name(const std::map<T, U>&) noexcept
{
    return "map";
}

template <typename T>
constexpr cstr_t type_name(const std::set<T>&) noexcept
{
    return "set";
}

template <typename T, size_t N>
constexpr cstr_t type_name(const std::array<T, N>&) noexcept
{
    return "array";
}
template <typename T>
constexpr cstr_t type_name(const std::vector<T>&) noexcept
{
    return "array";
}
template <typename T, size_t N>
constexpr cstr_t type_name(T (&)[N]) noexcept
{
    return "array";
}

template <typename T>
constexpr std::enable_if_t<std::is_enum_v<T>, cstr_t> type_name(T) noexcept
{
    return type_name(std::underlying_type_t<T>());
}

template <typename T>
constexpr std::enable_if_t<std::is_const_v<T> || std::is_volatile_v<T>, cstr_t> type_name() noexcept
{
    return type_name(std::remove_cv_t<T>());
}

} // namespace FLECS

#endif // B2BC57AF_1CAA_4C27_9C4F_940B766B7E3C
