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

#ifndef FLECS_util_map_constexpr_h
#define FLECS_util_map_constexpr_h

#include <algorithm>
#include <array>
#include <cassert>
#include <cstring>
#include <functional>
#include <stdexcept>
#include <utility>

namespace FLECS {

template <typename Key, typename Value, size_t size, typename Compare = std::less<Key>>
class map_c
{
public:
    using key_type = Key;
    using mapped_type = Value;
    using value_type = std::pair<const Key, Value>;
    using container_type = std::array<value_type, size>;
    using size_type = typename container_type::size_type;
    using difference_type = typename container_type::difference_type;
    using key_compare = Compare;
    using reference = value_type&;
    using const_reference = const value_type&;
    using pointer = value_type*;
    using const_pointer = const value_type*;
    using iterator = typename container_type::iterator;
    using const_iterator = typename container_type::const_iterator;
    using reverse_iterator = std::reverse_iterator<iterator>;
    using const_reverse_iterator = std::reverse_iterator<const_iterator>;

    constexpr map_c(container_type data)
        : _data{data}
    {}

    constexpr iterator begin() noexcept { return _data.begin(); }

    constexpr const_iterator begin() const noexcept { return _data.begin(); }

    constexpr const_iterator cbegin() const noexcept { return _data.cbegin(); }

    constexpr iterator end() noexcept { return _data.end(); }

    constexpr const_iterator end() const noexcept { return _data.end(); }

    constexpr const_iterator cend() const noexcept { return _data.cend(); }

    constexpr iterator find(const Key& key) noexcept
    {
        return std::find_if(_data.begin(), _data.end(), [&](const value_type& elem) { return equal(elem, key); });
    }

    constexpr const_iterator find(const Key& key) const noexcept
    {
        return std::find_if(_data.cbegin(), _data.cend(), [&](const value_type& elem) { return equal(elem, key); });
    }

    constexpr const_reference at(Key key) const
    {
        const auto it =
            std::find_if(_data.cbegin(), _data.cend(), [&](const value_type& elem) { return equal(elem, key); });

        if (it == _data.cend())
        {
            throw std::out_of_range{"key not found in map_constexpr"};
        }

        return *it;
    }

private:
    bool equal(const value_type& elem, const Key& key) const noexcept
    {
        return !Compare()(elem.first, key) && !Compare()(key, elem.first);
    }

    container_type _data;
};

} // namespace FLECS

#endif // FLECS_util_map_constexpr_h
