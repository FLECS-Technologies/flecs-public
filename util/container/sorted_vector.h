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

#ifndef D92E69C8_0D9E_4CCB_8A16_DC26BCD31521
#define D92E69C8_0D9E_4CCB_8A16_DC26BCD31521

#include <stdexcept>
#include <vector>

namespace FLECS {

template <typename Key, typename Compare = std::less<Key>, typename Alloc = std::allocator<Key>>
class sorted_vector_t
{
public:
    using key_type = typename std::vector<Key, Alloc>::value_type;
    using value_type = typename std::vector<Key, Alloc>::value_type;
    using key_compare = Compare;
    using value_compare = Compare;
    using allocator_type = typename std::vector<Key, Alloc>::allocator_type;
    using size_type = typename std::vector<Key, Alloc>::size_type;
    using difference_type = typename std::vector<Key, Alloc>::difference_type;
    using reference = typename std::vector<Key, Alloc>::reference;
    using const_reference = typename std::vector<Key, Alloc>::const_reference;
    using pointer = typename std::vector<Key, Alloc>::pointer;
    using const_pointer = typename std::vector<Key, Alloc>::const_pointer;
    using iterator = typename std::vector<Key, Alloc>::const_iterator;
    using const_iterator = typename std::vector<Key, Alloc>::const_iterator;
    using reverse_iterator = typename std::vector<Key, Alloc>::const_reverse_iterator;
    using const_reverse_iterator = typename std::vector<Key, Alloc>::const_reverse_iterator;

    sorted_vector_t()
        : sorted_vector_t(Compare())
    {}
    explicit sorted_vector_t(const Compare& comp, const Alloc& alloc = Alloc())
        : _vec{alloc}
        , _comp{comp}
    {}
    explicit sorted_vector_t(const Alloc& alloc)
        : sorted_vector_t(Compare(), alloc)
    {}

    template <class InputIt>
    sorted_vector_t(InputIt first, InputIt last, const Compare& comp, const Alloc& alloc = Alloc())
        : _vec{first, last, alloc}
        , _comp{comp}
    {
        sort_and_filter();
    }
    template <class InputIt>
    sorted_vector_t(InputIt first, InputIt last, const Alloc& alloc = Alloc())
        : sorted_vector_t(first, last, Compare(), alloc)
    {}

    sorted_vector_t(const sorted_vector_t& other, const Compare& comp, const Alloc& alloc = Alloc())
        : _vec{other._vec, alloc}
        , _comp{comp}
    {
        sort_and_filter();
    }
    sorted_vector_t(const sorted_vector_t& other, const Alloc& alloc = Alloc())
        : sorted_vector_t(other, Compare(), alloc)
    {}

    sorted_vector_t(sorted_vector_t&& other)
        : sorted_vector_t()
    {
        swap(*this, other);
    }
    sorted_vector_t(sorted_vector_t&& other, const Alloc& alloc)
        : _vec{std::move(other._vec), alloc}
        , _comp{std::move(other._comp)}
    {}

    explicit sorted_vector_t(std::initializer_list<key_type> init, const Alloc& alloc = Alloc())
        : sorted_vector_t(init, Compare(), alloc)
    {}
    sorted_vector_t(std::initializer_list<key_type> init, const Compare& comp, const Alloc& alloc = Alloc())
        : _vec{init, alloc}
        , _comp{comp}
    {
        sort_and_filter();
    }

    sorted_vector_t& operator=(sorted_vector_t other)
    {
        swap(*this, other);
        return *this;
    }
    sorted_vector_t& operator=(std::initializer_list<key_type> init)
    {
        return *this = sorted_vector_t<Key, Compare, Alloc>(init);
    }

    ~sorted_vector_t() = default;

    auto begin() noexcept -> iterator { return _vec.begin(); }
    auto begin() const noexcept -> const_iterator { return _vec.begin(); }
    auto cbegin() const noexcept -> const_iterator { return _vec.cbegin(); }

    auto end() noexcept -> iterator { return _vec.end(); }
    auto end() const noexcept -> const_iterator { return _vec.end(); }
    auto cend() const noexcept -> const_iterator { return _vec.cend(); }

    auto rbegin() noexcept -> reverse_iterator { return _vec.rbegin(); }
    auto rbegin() const noexcept -> const_reverse_iterator { return _vec.rbegin(); }
    auto crbegin() const noexcept -> const_reverse_iterator { return _vec.crbegin(); }

    auto rend() noexcept -> reverse_iterator { return _vec.rend(); }
    auto rend() const noexcept -> const_reverse_iterator { return _vec.rend(); }
    auto crend() const noexcept -> const_reverse_iterator { return _vec.crend(); }

    auto empty() const noexcept { return _vec.empty(); }
    auto size() const noexcept { return _vec.size(); }
    auto max_size() const noexcept { return _vec.max_size(); }
    auto reserve(size_type new_cap) { return _vec.reserve(new_cap); }
    auto capacity() const noexcept { return _vec.capacity(); }
    auto shrink_to_fit() { return _vec.shrink_to_fit(); }

    auto clear() { return _vec.clear(); }

    const_reference at(const key_type& key) const { return at(std::move(key)); }
    template <typename K>
    const_reference at(K&& x) const
    {
        const auto it = find(x);
        if (__glibc_unlikely(it == cend())) {
            throw std::out_of_range{"sorted_vector subscript out of range"};
        }
        return *it;
    }

    auto insert(const value_type& value) { return insert_impl(value); }
    auto insert(value_type&& value) { return insert_impl(std::move(value)); }
    auto insert(const_iterator hint, const value_type& value) { return insert_impl(hint, value); }
    auto insert(const_iterator hint, value_type&& value) { return insert_impl(hint, std::move(value)); }
    template <class InputIt>
    auto insert(InputIt first, InputIt last)
    {
        for (; first != last; insert(*first++))
            ;
    }
    auto insert(std::initializer_list<value_type> init) { return insert(init.begin(), init.end()); }

    template <typename... Args>
    auto emplace(Args&&... args)
    {
        return insert(value_type(std::forward<Args>(args)...));
    }
    template <typename... Args>
    auto emplace_hint(const_iterator hint, Args&&... args)
    {
        return insert(hint, value_type(std::forward<Args>(args)...));
    }

    auto erase(const_iterator pos) { return erase(pos, pos + 1); }
    auto erase(const_iterator first, const_iterator last) { return _vec.erase(first, last); }
    auto erase(const Key& key) { return erase(std::move(key)); }
    template <typename K>
    auto erase(K&& x)
    {
        if (!contains(std::forward<K>(x))) {
            return static_cast<size_type>(0);
        }
        static_cast<void>(erase(find(x)));
        return static_cast<size_type>(1);
    }

    auto count(const key_type& key) const { return count(std::move(key)); }
    template <typename K>
    auto count(K&& x) const
    {
        return static_cast<size_type>(contains(std::forward<K>(x)));
    }

    auto contains(const key_type& key) const { return contains(std::move(key)); }
    template <typename K>
    auto contains(K&& x) const
    {
        return find(std::forward<K>(x)) != cend();
    }

    auto find(const key_type& key) { return find(std::move(key)); }
    auto find(const key_type& key) const { return find(std::move(key)); }
    template <typename K>
    auto find(K&& x)
    {
        const auto range = equal_range(std::forward<K>(x));
        // if the first element not less than <x> is also greater than <x>, we do not have a match
        return (range.first == range.second) ? end() : range.first;
    }
    template <typename K>
    auto find(K&& x) const
    {
        return static_cast<const_iterator>((const_cast<this_type*>(this))->find(std::forward<K>(x)));
    }

    auto lower_bound(const key_type& key) { return lower_bound(std::move(key)); }
    auto lower_bound(const key_type& key) const { return lower_bound(std::move(key)); }
    template <typename K>
    auto lower_bound(K&& x)
    {
        return lower_bound_impl(begin(), end(), std::forward<K>(x));
    }
    template <typename K>
    auto lower_bound(K&& x) const
    {
        return static_cast<const_iterator>((const_cast<this_type*>(this))->lower_bound(std::forward<K>(x)));
    }

    auto upper_bound(const key_type& key) { return upper_bound(std::move(key)); }
    auto upper_bound(const key_type& key) const { return upper_bound(std::move(key)); }
    template <typename K>
    auto upper_bound(K&& x)
    {
        return upper_bound_impl(begin(), end(), std::forward<K>(x));
    }
    template <typename K>
    auto upper_bound(K&& x) const
    {
        return static_cast<const_iterator>((const_cast<this_type*>(this))->upper_bound(std::forward<K>(x)));
    }

    auto equal_range(const key_type& key) { return equal_range(std::move(key)); }
    auto equal_range(const key_type& key) const { return equal_range(std::move(key)); }
    template <typename K>
    auto equal_range(K&& x)
    {
        const auto l = lower_bound(std::forward<K>(x));
        return std::make_pair(l, upper_bound_impl(l, end(), std::forward<K>(x)));
    }
    template <typename K>
    auto equal_range(K&& x) const
    {
        const auto range = (const_cast<this_type*>(this))->equal_range(std::forward<K>(x));
        return std::make_pair(static_cast<const_iterator>(range.first), static_cast<const_iterator>(range.second));
    }

    auto key_comp() const { return value_comp(); }
    auto value_comp() const { return _comp; }

private:
    using this_type = sorted_vector_t<Key, Compare, Alloc>;

    template <typename T>
    auto is_insertable(T&& value) //
        -> std::tuple<iterator, bool>;

    template <typename T>
    auto insert_impl(T&& value) //
        -> std::pair<iterator, bool>;

    template <typename T>
    auto insert_impl(const_iterator hint, T&& value) //
        -> iterator;

    template <typename T>
    auto emplace_impl(T&& value) //
        -> std::pair<iterator, bool>;

    template <typename T>
    auto emplace_impl(const_iterator hint, T&& value) //
        -> std::pair<iterator, bool>;

    template <typename K>
    auto lower_bound_impl(iterator first, iterator last, K&& x)
    {
        return std::lower_bound(first, last, std::forward<K>(x));
    }

    template <typename K>
    auto upper_bound_impl(iterator first, iterator last, K&& x)
    {
        return std::upper_bound(first, last, std::forward<K>(x));
    }

    auto sort_and_filter()
    {
        std::sort(_vec.begin(), _vec.end());
        _vec.erase(std::unique(_vec.begin(), _vec.end()), _vec.end());
    }

    friend auto swap(sorted_vector_t& lhs, sorted_vector_t& rhs) //
        -> void
    {
        using std::swap;
        swap(lhs._vec, rhs._vec);
        swap(lhs._comp, rhs._comp);
    }

    std::vector<Key, Alloc> _vec;
    Compare _comp;
};

template <typename Key, typename Compare, typename Alloc>
template <typename T>
auto sorted_vector_t<Key, Compare, Alloc>::insert_impl(T&& value) -> //
    std::pair<iterator, bool>
{
    const auto it = lower_bound(value);
    // either there is no element greater than <value>, or <it> points to element greater than <value>
    if ((it == cend()) || key_comp()(value, *it)) {
        return std::pair<iterator, bool>{_vec.insert(it, std::forward<T>(value)), true};
    }
    return std::pair<iterator, bool>{it, false};
}

template <typename Key, typename Compare, typename Alloc>
template <typename T>
auto sorted_vector_t<Key, Compare, Alloc>::insert_impl(const_iterator hint, T&& value) -> //
    iterator
{
    if (hint == cend()) {
        // check if cend() hint is correct
        if (!empty() && key_comp()(*rbegin(), value)) {
            return _vec.insert(hint, std::forward<T>(value));
        }
    } else {
        // check if other hint is correct
        auto pos = hint;
        // hint points to element after
        if (key_comp()(value, *pos) && ((--pos == cbegin()) || (key_comp()(*pos, value)))) {
            return _vec.insert(hint, std::forward<T>(value));
        }

        pos = hint;
        // hint points to element before
        if (key_comp()(*pos, value) && ((++pos == cend()) || (key_comp()(value, *pos)))) {
            return _vec.insert(++hint, std::forward<T>(value));
        }
    }
    return insert_impl(std::forward<T>(value)).first;
}

} // namespace FLECS

#endif /* D92E69C8_0D9E_4CCB_8A16_DC26BCD31521 */
