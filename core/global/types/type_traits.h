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

#ifndef EE7E8533_2B64_4B48_AE6F_802059D018CF
#define EE7E8533_2B64_4B48_AE6F_802059D018CF

#include <string>
#include <type_traits>

#include "core/global/types/types.h"

namespace FLECS {

// is_std_string and is_std_string_v
template <typename T>
struct is_std_string_helper : std::false_type
{
};

template <typename CharType, typename Traits, typename Allocator>
struct is_std_string_helper<std::basic_string<CharType, Traits, Allocator>> : std::true_type
{
};

template <typename T>
struct is_std_string : public is_std_string_helper<std::decay_t<T>>
{
};

template <typename T>
inline constexpr bool is_std_string_v = is_std_string<T>::value;

// is_std_string_view and is_std_string_view_v
template <typename T>
struct is_std_string_view_helper : std::false_type
{
};

template <typename CharType, typename Traits>
struct is_std_string_view_helper<std::basic_string_view<CharType, Traits>> : std::true_type
{
};

template <typename T>
struct is_std_string_view : public is_std_string_view_helper<std::decay_t<T>>
{
};

template <typename T>
inline constexpr bool is_std_string_view_v = is_std_string_view<T>::value;

// is_std_container and is_std_container_v
// std::string and std::string_view are intentionally excluded, although they technically fulfill the basic Container
// concept. However, the focus of is_std_container lies on containers without limitation of or assumptions about the
// contained type.
template <typename T, typename = void>
struct is_std_container : std::false_type
{
};

template <typename ContainerType>
struct is_std_container<
    ContainerType,
    std::enable_if_t<
        !is_std_string_v<ContainerType> && !is_std_string_view_v<ContainerType>,
        std::void_t<
            typename std::decay_t<ContainerType>::value_type,
            typename std::decay_t<ContainerType>::reference,
            typename std::decay_t<ContainerType>::const_reference,
            typename std::decay_t<ContainerType>::iterator,
            typename std::decay_t<ContainerType>::const_iterator,
            typename std::decay_t<ContainerType>::difference_type,
            typename std::decay_t<ContainerType>::size_type,
            decltype(std::decay_t<ContainerType>().begin()),
            decltype(std::decay_t<ContainerType>().end()),
            decltype(std::decay_t<ContainerType>().cbegin()),
            decltype(std::decay_t<ContainerType>().cend()),
            decltype(std::decay_t<ContainerType>().size()),
            decltype(std::decay_t<ContainerType>().empty())>>> : std::true_type
{
};

template <typename T>
inline constexpr bool is_std_container_v = is_std_container<T>::value;

} // namespace FLECS

#endif // EE7E8533_2B64_4B48_AE6F_802059D018CF
