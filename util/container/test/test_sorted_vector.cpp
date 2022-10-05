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

#include <sys/resource.h>

#include "gtest/gtest.h"
#include "util/container/sorted_vector.h"

class non_copyable_t
{
public:
    non_copyable_t(int n)
        : _n{n}
    {}
    non_copyable_t(const non_copyable_t&) = delete;
    non_copyable_t(non_copyable_t&&) = default;
    non_copyable_t& operator=(const non_copyable_t&) = delete;
    non_copyable_t& operator=(non_copyable_t&&) = default;
    ~non_copyable_t() = default;

private:
    friend auto operator<(const non_copyable_t& lhs, const non_copyable_t& rhs) { return lhs._n < rhs._n; }
    friend auto operator==(const non_copyable_t& lhs, const non_copyable_t& rhs)
    {
        return !(lhs < rhs) && !(rhs < lhs);
    }

    int _n;
};

template <typename Key, typename Compare, typename Alloc>
void assert_is_sorted(const FLECS::sorted_vector_t<Key, Compare, Alloc>& uut)
{
    ASSERT_FALSE(uut.empty());

    auto prev = *uut.cbegin();
    for (auto it = uut.cbegin() + 1; it != uut.cend(); ++it)
    {
        ASSERT_TRUE(Compare()(prev, *it));
        prev = *it;
    }
}

TEST(sorted_vector, empty)
{
    const auto uut = FLECS::sorted_vector_t<int>{};

    ASSERT_EQ(uut.size(), 0);
}

template <typename T>
void sorted_vector_init_test(T&& uut)
{
    ASSERT_EQ(uut.begin(), uut.cbegin());
    ASSERT_EQ(uut.end(), uut.cend());
    ASSERT_EQ(uut.rbegin(), uut.crbegin());
    ASSERT_EQ(uut.rend(), uut.crend());

    ASSERT_FALSE(uut.empty());
    ASSERT_EQ(uut.size(), 6);

    ASSERT_EQ(uut.count(0), 1);
    ASSERT_EQ(uut.count(0LL), 1);
    ASSERT_TRUE(uut.contains(0));
    ASSERT_TRUE(uut.contains(0LL));
    ASSERT_EQ(uut.find(0), uut.begin());
    ASSERT_EQ(uut.find(0LL), uut.begin());

    const int32_t i32_0 = 0;
    const int64_t i64_0 = 0LL;

    ASSERT_EQ(uut.at(i32_0), 0);
    ASSERT_EQ(uut.at(i64_0), 0);
    ASSERT_ANY_THROW(uut.at(6));

    ASSERT_EQ(uut.count(i32_0), 1);
    ASSERT_EQ(uut.count(i64_0), 1);
    ASSERT_TRUE(uut.contains(i32_0));
    ASSERT_TRUE(uut.contains(i64_0));
    ASSERT_EQ(uut.find(i32_0), uut.begin());
    ASSERT_EQ(uut.find(i64_0), uut.begin());

    const int32_t i32_5 = 5;
    const int64_t i64_5 = 5LL;

    ASSERT_EQ(uut.lower_bound(0), uut.cbegin());
    ASSERT_EQ(uut.lower_bound(0LL), uut.cbegin());
    ASSERT_EQ(uut.lower_bound(i32_5), uut.cend() - 1);
    ASSERT_EQ(uut.lower_bound(i64_5), uut.cend() - 1);
    ASSERT_EQ(uut.lower_bound(6), uut.cend());
    ASSERT_EQ(uut.lower_bound(6LL), uut.cend());
    ASSERT_EQ(uut.upper_bound(0), uut.cbegin() + 1);
    ASSERT_EQ(uut.upper_bound(0LL), uut.cbegin() + 1);
    ASSERT_EQ(uut.upper_bound(4), uut.cend() - 1);
    ASSERT_EQ(uut.upper_bound(4LL), uut.cend() - 1);
    ASSERT_EQ(uut.upper_bound(i32_5), uut.cend());
    ASSERT_EQ(uut.upper_bound(i64_5), uut.cend());

    ASSERT_EQ(uut.equal_range(0).first, uut.lower_bound(0));
    ASSERT_EQ(uut.equal_range(0LL).first, uut.lower_bound(0LL));
    ASSERT_EQ(uut.equal_range(0).second, uut.upper_bound(0));
    ASSERT_EQ(uut.equal_range(0LL).second, uut.upper_bound(0LL));

    ASSERT_EQ(uut.equal_range(i32_5).first, uut.lower_bound(5));
    ASSERT_EQ(uut.equal_range(i64_5).first, uut.lower_bound(5LL));
    ASSERT_EQ(uut.equal_range(i32_5).second, uut.upper_bound(5));
    ASSERT_EQ(uut.equal_range(i64_5).second, uut.upper_bound(5LL));

    auto i = 0;
    for (const auto& it : uut)
    {
        ASSERT_EQ(it, i);
        ++i;
    }
}

TEST(sorted_vector, init)
{
    auto uut = FLECS::sorted_vector_t<int>{5, 5, 4, 3, 2, 1, 0, 0};
    const auto& uut_c = uut;

    // non-const
    sorted_vector_init_test(uut);
    // const
    sorted_vector_init_test(uut_c);
}

TEST(sorted_vector, insert)
{
    auto uut = FLECS::sorted_vector_t<int>{5, 5, 4, 3, 2, 1, 0, 0};

    const auto res1 = uut.insert(5);
    ASSERT_EQ(uut.size(), 6);
    ASSERT_EQ(res1.first, uut.find(5));
    ASSERT_FALSE(res1.second);

    const auto res2 = uut.insert(6);
    ASSERT_EQ(uut.size(), 7);
    ASSERT_EQ(res2.first, uut.find(6));
    ASSERT_TRUE(res2.second);

    const auto res3 = uut.insert(6);
    ASSERT_EQ(uut.size(), 7);
    ASSERT_EQ(res2.first, res3.first);
    ASSERT_FALSE(res3.second);

    const auto i32 = 7;
    const auto res4 = uut.insert(i32);
    ASSERT_EQ(uut.size(), 8);
    ASSERT_EQ(res4.first, uut.find(i32));
    ASSERT_TRUE(res4.second);

    const auto res5 = uut.insert(i32);
    ASSERT_EQ(uut.size(), 8);
    ASSERT_EQ(res5.first, res4.first);
    ASSERT_FALSE(res5.second);

    uut.insert({0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10});
    ASSERT_EQ(uut.size(), 11);

    assert_is_sorted(uut);
}

TEST(sorted_vector, insert_hint)
{
    auto uut = FLECS::sorted_vector_t<int>{5, 5, 4, 3, 2, 1, 0, 0};

    // iterator, value&&
    const auto it1 = uut.insert(uut.end(), 5);
    ASSERT_EQ(uut.size(), 6);
    ASSERT_EQ(it1, uut.find(5));

    const auto it2 = uut.insert(uut.end(), 5);
    ASSERT_EQ(uut.size(), 6);
    ASSERT_EQ(it2, uut.find(5));

    // const_iterator, value&&
    const auto it3 = uut.insert(uut.cend(), 8);
    ASSERT_EQ(uut.size(), 7);
    ASSERT_EQ(it3, uut.find(8));

    const auto it4 = uut.insert(uut.cend(), 8);
    ASSERT_EQ(uut.size(), 7);
    ASSERT_EQ(it4, uut.find(8));

    // iterator, const value&
    const auto i32 = 6;
    const auto it5 = uut.insert(uut.find(8), i32);
    ASSERT_EQ(uut.size(), 8);
    ASSERT_EQ(it5, uut.find(6));

    const auto it6 = uut.insert(uut.find(8), i32);
    ASSERT_EQ(uut.size(), 8);
    ASSERT_EQ(it6, uut.find(6));

    assert_is_sorted(uut);
}

TEST(sorted_vector, emplace)
{
    ASSERT_FALSE(std::is_copy_assignable_v<non_copyable_t>);
    ASSERT_FALSE(std::is_copy_constructible_v<non_copyable_t>);

    auto uut = FLECS::sorted_vector_t<non_copyable_t>{};
    uut.emplace(3);

    ASSERT_EQ(uut.size(), 1);
    ASSERT_EQ(*uut.begin(), 3);
}

TEST(sorted_vector, emplace_hint)
{
    ASSERT_FALSE(std::is_copy_assignable_v<non_copyable_t>);
    ASSERT_FALSE(std::is_copy_constructible_v<non_copyable_t>);

    auto uut = FLECS::sorted_vector_t<non_copyable_t>{};
    auto elem = non_copyable_t{1};
    uut.emplace_hint(uut.cend(), 3);
    uut.emplace_hint(uut.cend(), non_copyable_t{4});
    uut.emplace_hint(uut.cbegin(), std::move(elem));

    ASSERT_EQ(uut.size(), 3);
    ASSERT_EQ(*uut.begin(), 1);
}

TEST(sorted_vector, erase)
{
    auto uut = FLECS::sorted_vector_t<int>{5, 5, 4, 3, 2, 1, 0, 0};

    uut.erase(uut.begin());
    ASSERT_EQ(uut.size(), 5);
    uut.erase(uut.cbegin());
    ASSERT_EQ(uut.size(), 4);
    uut.erase(uut.cbegin(), uut.cbegin() + 2);
    ASSERT_EQ(uut.size(), 2);
    auto res = uut.erase(5);
    ASSERT_EQ(res, 1);
    ASSERT_EQ(uut.size(), 1);
    const auto i32_5 = 5;
    res = uut.erase(i32_5);
    ASSERT_EQ(res, 0);
    ASSERT_EQ(uut.size(), 1);
}

TEST(sorted_vector, capacity)
{
    auto uut = FLECS::sorted_vector_t<int>{};

    uut.reserve(16);
    ASSERT_EQ(uut.capacity(), 16);

    uut.insert(1);
    uut.shrink_to_fit();
    ASSERT_EQ(uut.capacity(), 1);

    ASSERT_EQ(uut.size(), 1);
    uut.clear();
    ASSERT_EQ(uut.size(), 0);
}

TEST(sorted_vector, copy_and_move)
{
    auto uut1 = FLECS::sorted_vector_t<int>{8, 7, 6, 5, 4, 3, 2, 1};

    // copy constructor
    auto uut2 = uut1;
    ASSERT_TRUE(std::equal(uut1.cbegin(), uut1.cend(), uut2.cbegin()));

    // move constructor
    auto uut3 = std::move(uut1);
    ASSERT_TRUE(uut1.empty());
    ASSERT_EQ(uut3.size(), 8);

    // range constructor
    auto uut4 = FLECS::sorted_vector_t<int>{uut2.cbegin(), uut2.cend()};
    ASSERT_TRUE(std::equal(uut4.cbegin(), uut4.cend(), uut2.cbegin()));

    // copy assignment
    auto uut5 = FLECS::sorted_vector_t<int>{};
    uut5 = uut2;
    ASSERT_TRUE(std::equal(uut5.cbegin(), uut5.cend(), uut2.cbegin()));

    const auto uut6_init = std::initializer_list<int>{0, 1, 2, 3, 4};
    auto uut6 = FLECS::sorted_vector_t<int>{};
    uut6 = uut6_init;
    ASSERT_TRUE(std::equal(uut6.cbegin(), uut6.cend(), uut6_init.begin()));

    // move assignment
    auto uut7 = FLECS::sorted_vector_t<int>{};
    uut7 = std::move(uut3);
    ASSERT_TRUE(uut3.empty());
    ASSERT_EQ(uut7.size(), 8);
}

TEST(sorted_vector, performance_1)
{
    auto uut_1 = FLECS::sorted_vector_t<int>{};
    auto uut_2 = FLECS::sorted_vector_t<int>{};

    uut_1.reserve(1024 * 1024 * 4);
    uut_2.reserve(1024 * 1024 * 4);

    auto usage = rusage{};

    getrusage(RUSAGE_SELF, &usage);
    const auto t1 = usage.ru_utime;
    for (std::size_t i = 0; i < uut_1.capacity(); ++i)
    {
        uut_1.insert(i);
    }

    getrusage(RUSAGE_SELF, &usage);
    const auto t2 = usage.ru_utime;
    for (std::size_t i = 0; i < uut_2.capacity(); ++i)
    {
        uut_2.insert(uut_2.cend(), i);
    }

    getrusage(RUSAGE_SELF, &usage);
    const auto t3 = usage.ru_utime;

    const auto d1 = (t2.tv_sec - t1.tv_sec) * 1000000 + (t2.tv_usec - t1.tv_usec);
    const auto d2 = (t3.tv_sec - t2.tv_sec) * 1000000 + (t3.tv_usec - t2.tv_usec);

    ASSERT_EQ(uut_1.size(), uut_2.size());
    ASSERT_TRUE(std::equal(uut_1.cbegin(), uut_1.cend(), uut_2.cbegin()));
    ASSERT_GT(d1, d2);
}

#if 0  // Reevaluate
// Insertion overhead is too high to deterministically be faster with hinted insertion
TEST(sorted_vector, performance_2)
{
    auto uut_1 = FLECS::sorted_vector_t<int>{};
    auto uut_2 = FLECS::sorted_vector_t<int>{};

    uut_1.reserve(1024 * 1024);
    uut_2.reserve(1024 * 1024);

    for (std::size_t i = 0; i < uut_1.capacity() / 2; ++i)
    {
        uut_1.insert(i);
    }
    for (std::size_t i = 0; i < uut_2.capacity() / 2; ++i)
    {
        uut_2.insert(uut_2.cend(), i);
    }

    auto usage = rusage{};

    getrusage(RUSAGE_SELF, &usage);
    const auto t1 = usage.ru_utime;
    for (std::size_t i = uut_1.capacity() / 2; i < uut_1.capacity(); ++i)
    {
        uut_1.insert(std::numeric_limits<int>::max() - i);
    }

    getrusage(RUSAGE_SELF, &usage);
    const auto t2 = usage.ru_utime;
    const auto it = uut_2.cbegin() + uut_2.capacity() / 2;
    for (std::size_t i = uut_2.capacity() / 2; i < uut_2.capacity(); ++i)
    {
        uut_2.insert(it, std::numeric_limits<int>::max() - i);
    }

    getrusage(RUSAGE_SELF, &usage);
    const auto t3 = usage.ru_utime;

    const auto d1 = (t2.tv_sec - t1.tv_sec) * 1000000 + (t2.tv_usec - t1.tv_usec);
    std::fprintf(stdout, "raw: %ld usec\n", d1);
    const auto d2 = (t3.tv_sec - t2.tv_sec) * 1000000 + (t3.tv_usec - t2.tv_usec);
    std::fprintf(stdout, "hint: %ld usec\n", d2);

    ASSERT_EQ(uut_1.size(), uut_2.size());
    ASSERT_TRUE(std::equal(uut_1.cbegin(), uut_1.cend(), uut_2.cbegin()));
    ASSERT_GT(d1, d2);
}
#endif // 0
