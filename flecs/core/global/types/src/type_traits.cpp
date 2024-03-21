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

#include "flecs/core/global/types/type_traits.h"

#include <array>
#include <deque>
#include <list>
#include <map>
#include <set>
#include <string>
#include <string_view>
#include <unordered_map>
#include <unordered_set>
#include <vector>

namespace flecs {

// static_assert is_std_string
static_assert(is_std_string_v<std::string>);
static_assert(is_std_string_v<std::wstring>);
static_assert(!is_std_string_v<std::string_view>);
static_assert(!is_std_string_v<std::wstring_view>);

// static_assert is_std_string_view
static_assert(!is_std_string_view_v<std::string>);
static_assert(!is_std_string_view_v<std::wstring>);
static_assert(is_std_string_view_v<std::string_view>);
static_assert(is_std_string_view_v<std::wstring_view>);

// static_assert is_std_container_v
static_assert(is_std_container_v<std::array<int8_t, 1>>);
static_assert(is_std_container_v<std::deque<int8_t>>);
static_assert(is_std_container_v<std::list<int8_t>>);
static_assert(is_std_container_v<std::map<int8_t, int8_t>>);
static_assert(is_std_container_v<std::multimap<int8_t, int8_t>>);
static_assert(is_std_container_v<std::set<int8_t>>);
static_assert(is_std_container_v<std::multiset<int8_t>>);
static_assert(!is_std_container_v<std::string>);
static_assert(!is_std_container_v<std::wstring>);
static_assert(!is_std_container_v<std::string_view>);
static_assert(!is_std_container_v<std::wstring_view>);
static_assert(is_std_container_v<std::unordered_map<int8_t, int8_t>>);
static_assert(is_std_container_v<std::unordered_multimap<int8_t, int8_t>>);
static_assert(is_std_container_v<std::unordered_set<int8_t>>);
static_assert(is_std_container_v<std::unordered_multiset<int8_t>>);
static_assert(is_std_container_v<std::vector<int8_t>>);

} // namespace flecs
