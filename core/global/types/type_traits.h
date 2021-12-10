// Copyright 2021 FLECS Technologies GmbH
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
#ifndef FLECS_core_global_types_type_traits_h
#define FLECS_core_global_types_type_traits_h

#include "core/global/types/types.h"

#include <string>
#include <type_traits>

namespace FLECS {

// is_std_string and is_std_string_v
template <typename T>
struct is_std_string : std::false_type {};

template <typename CharType, typename Traits, typename Allocator>
struct is_std_string<std::basic_string<CharType, Traits, Allocator>> : std::true_type {};

template <typename T>
inline constexpr bool is_std_string_v = is_std_string<T>::value;

// is_std_string_view and is_std_string_view_v
template <typename T>
struct is_std_string_view : std::false_type {};

template <typename CharType, typename Traits>
struct is_std_string_view<std::basic_string_view<CharType, Traits>> : std::true_type {};

template <typename T>
inline constexpr bool is_std_string_view_v = is_std_string_view<T>::value;

// is_std_container and is_std_container_v
// std::string and std::string_view are intentionally excluded, although they technically fulfill the basic Container
// concept. However, the focus of is_std_container lies on containers without limitation of or assumptions about the
// contained type.
template <typename T, typename = void>
struct is_std_container : std::false_type {};

template <typename ContainerType>
struct is_std_container<ContainerType,
    std::enable_if_t<
        !is_std_string_v<ContainerType> &&
        !is_std_string_view_v<ContainerType>,
        std::void_t<
            typename ContainerType::value_type,
            typename ContainerType::reference,
            typename ContainerType::const_reference,
            typename ContainerType::iterator,
            typename ContainerType::const_iterator,
            typename ContainerType::difference_type,
            typename ContainerType::size_type,
            decltype(((ContainerType*)nullptr)->begin()),
            decltype(((ContainerType*)nullptr)->end()),
            decltype(((ContainerType*)nullptr)->cbegin()),
            decltype(((ContainerType*)nullptr)->cend()),
            decltype(((ContainerType*)nullptr)->size()),
            decltype(((ContainerType*)nullptr)->empty())
        >
    >
> : std::true_type {};

template <typename T>
inline constexpr bool is_std_container_v = is_std_container<T>::value;

} // namespace FLECS

#endif // FLECS_core_global_types_type_traits_h
