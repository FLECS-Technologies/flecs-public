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

#include <gtest/gtest.h>
#include <zenoh.h>

#include <condition_variable>
#include <mutex>
#include <thread>

#include "core/global/types/type_traits.h"
#include "flunder/flunder_client.h"
#include "util/string/string_utils.h"

static auto cv = std::condition_variable{};
static auto m = std::mutex{};
static auto done = false;

struct custom_t
{
};

template <typename T>
auto topic(T);

#define TOPIC_BASE "/flecs/flunder/test/"
#define DEF_TOPIC(type)          \
    template <>                  \
    auto topic<type>(type)       \
    {                            \
        return TOPIC_BASE #type; \
    }
#define DEF_TOPIC_EX(type, suffix) \
    template <>                    \
    auto topic<type>(type)         \
    {                              \
        return TOPIC_BASE #suffix; \
    }

DEF_TOPIC(std::int8_t)
DEF_TOPIC(std::int16_t)
DEF_TOPIC(std::int32_t)
DEF_TOPIC(std::int64_t)
DEF_TOPIC(std::uint8_t)
DEF_TOPIC(std::uint16_t)
DEF_TOPIC(std::uint32_t)
DEF_TOPIC(std::uint64_t)
DEF_TOPIC(bool)
DEF_TOPIC(float)
DEF_TOPIC(double)
DEF_TOPIC(std::string)
DEF_TOPIC(std::string_view)
DEF_TOPIC_EX(nullptr_t, none)
DEF_TOPIC_EX(void*, raw)
DEF_TOPIC_EX(custom_t, custom)
DEF_TOPIC_EX(const char*, string)

template <typename T>
constexpr auto encoding(T);

template <typename T>
constexpr auto is_signed(T)
{
    if constexpr (std::is_signed_v<T>)
    {
        return "s";
    }
    return "u";
}

#define DEF_APP_ENCODING(type)          \
    template <>                         \
    constexpr auto encoding<type>(type) \
    {                                   \
        return "application/" #type;    \
    }
#define DEF_INT_ENCODING(type)                                                                             \
    template <>                                                                                            \
    auto encoding<type>(type)                                                                              \
    {                                                                                                      \
        using std::operator""s;                                                                            \
        return "application/integer+"s.append(is_signed(type{})).append(std::to_string(8 * sizeof(type))); \
    }
#define DEF_FLOAT_ENCODING(type)                                               \
    template <>                                                                \
    auto encoding<type>(type)                                                  \
    {                                                                          \
        using std::operator""s;                                                \
        return "application/float+"s.append(std::to_string(8 * sizeof(type))); \
    }
#define DEF_CUSTOM_ENCODING(type, enc) \
    template <>                        \
    auto encoding<type>(type)          \
    {                                  \
        return enc;                    \
    }
DEF_APP_ENCODING(bool)
DEF_INT_ENCODING(std::int8_t)
DEF_INT_ENCODING(std::int16_t)
DEF_INT_ENCODING(std::int32_t)
DEF_INT_ENCODING(std::int64_t)
DEF_INT_ENCODING(std::uint8_t)
DEF_INT_ENCODING(std::uint16_t)
DEF_INT_ENCODING(std::uint32_t)
DEF_INT_ENCODING(std::uint64_t)
DEF_FLOAT_ENCODING(float);
DEF_FLOAT_ENCODING(double);
DEF_CUSTOM_ENCODING(std::string, "text/plain")
DEF_CUSTOM_ENCODING(std::string_view, "text/plain")
DEF_CUSTOM_ENCODING(const char*, "text/plain")
DEF_CUSTOM_ENCODING(void*, "application/octet-stream")
DEF_CUSTOM_ENCODING(custom_t, "my-type")

template <typename T>
auto val(T) -> std::enable_if_t<std::is_integral_v<T> && !std::is_same_v<T, bool>, T>
{
    return T{123};
}
template <typename T>
auto val(T) -> std::enable_if_t<std::is_integral_v<T> && std::is_same_v<T, bool>, T>
{
    return T{true};
}
template <typename T>
auto val(T) -> std::enable_if_t<std::is_floating_point_v<T>, T>
{
    return T{3.14159};
}
auto val(std::string)
{
    using std::operator""s;
    return "Hello, FLECS!"s;
}
auto val(std::string_view)
{
    using std::operator""s;
    return "Hello, FLECS!"s;
}
auto val(const char*)
{
    return "Hello, FLECS!";
}

template <typename T>
void flunder_cbk_userp(FLECS::flunder_client_t* client, const FLECS::flunder_variable_t* var, const void* userp)
{
    std::fprintf(stderr, "Received topic %s\n", var->topic().data());
    ASSERT_EQ(client, userp);
    ASSERT_EQ(var->encoding(), encoding(T{}));
    ASSERT_EQ(var->topic(), topic(T{}));
    ASSERT_EQ(var->len(), FLECS::stringify(val(T{})).length());
    ASSERT_EQ(var->value(), FLECS::stringify(val(T{})));
}

void flunder_cbk(FLECS::flunder_client_t* /*client*/, const FLECS::flunder_variable_t* var)
{
    std::fprintf(stderr, "Received topic %s\n", var->topic().data());
    if (var->topic() == topic((void*)(nullptr)))
    {
        ASSERT_EQ(var->encoding(), encoding((void*)(nullptr)));
    }
    else if (var->topic() == topic(custom_t{}))
    {
        ASSERT_EQ(var->encoding(), encoding(custom_t{}));
        auto lock_guard = std::lock_guard<std::mutex>{m};
        done = true;
        cv.notify_all();
    }
}

TEST(flunder, init)
{
    auto client_1 = FLECS::flunder_client_t{};
    ASSERT_FALSE(client_1.is_connected());

    auto res = client_1.connect("172.17.0.1", 7447);
    ASSERT_EQ(res, 0);
    ASSERT_TRUE(client_1.is_connected());

    auto client_2 = FLECS::flunder_client_t{std::move(client_1)};
    ASSERT_FALSE(client_1.is_connected());
    ASSERT_TRUE(client_2.is_connected());

    auto client_3 = FLECS::flunder_client_t{};
    client_3 = std::move(client_2);
    ASSERT_FALSE(client_1.is_connected());
    ASSERT_FALSE(client_2.is_connected());
    ASSERT_TRUE(client_3.is_connected());

    res = client_3.reconnect();
    ASSERT_EQ(res, 0);
    ASSERT_TRUE(client_3.is_connected());

    res = client_3.disconnect();
    ASSERT_EQ(res, 0);
    ASSERT_FALSE(client_3.is_connected());
}

TEST(flunder, pub_sub)
{
    auto client_1 = FLECS::flunder_client_t{};
    auto client_2 = FLECS::flunder_client_t{};

    client_1.connect("172.17.0.1", 7447);
    client_2.connect("172.17.0.1", 7447);

    auto res =
        client_1.subscribe(topic(nullptr), flunder_cbk_userp<std::int32_t>, reinterpret_cast<const void*>(&client_1));
    ASSERT_EQ(res, 0);

    res = client_1.subscribe(
        topic(std::string{}),
        flunder_cbk_userp<std::string>,
        reinterpret_cast<const void*>(&client_1));
    ASSERT_EQ(res, 0);

    res = client_1.subscribe(
        topic(std::string_view{}),
        flunder_cbk_userp<std::string_view>,
        reinterpret_cast<const void*>(&client_1));
    ASSERT_EQ(res, 0);

    res = client_1.subscribe(
        topic(std::int32_t{}),
        flunder_cbk_userp<std::int32_t>,
        reinterpret_cast<const void*>(&client_1));
    ASSERT_EQ(res, 0);

    res = client_1.subscribe(topic(bool{}), flunder_cbk_userp<bool>, reinterpret_cast<const void*>(&client_1));
    ASSERT_EQ(res, 0);

    res = client_1.subscribe(topic(float{}), flunder_cbk_userp<float>, reinterpret_cast<const void*>(&client_1));
    ASSERT_EQ(res, 0);

    res = client_1.subscribe(topic(double{}), flunder_cbk_userp<double>, reinterpret_cast<const void*>(&client_1));
    ASSERT_EQ(res, 0);

    res = client_1.subscribe(topic((void*)(nullptr)), &flunder_cbk);
    ASSERT_EQ(res, 0);

    res = client_1.subscribe(topic(custom_t{}), &flunder_cbk);
    ASSERT_EQ(res, 0);
    /* attempt to subscribe again */
    res = client_1.subscribe(topic(custom_t{}), &flunder_cbk);
    ASSERT_EQ(res, -1);

    res = client_1.unsubscribe(topic(nullptr));
    ASSERT_EQ(res, 0);

    using std::operator""sv;
    res = client_2.publish(topic(nullptr), "Hello, FLECS!");
    ASSERT_EQ(res, 0);
    usleep(250);
    res = client_2.publish(topic(std::string{}), val(std::string{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = client_2.publish(topic(std::string_view{}), val(std::string_view{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = client_2.publish(topic(int32_t{}), val(std::int32_t{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = client_2.publish(topic(bool{}), true);
    ASSERT_EQ(res, 0);
    usleep(250);
    res = client_2.publish(topic(float{}), val(float{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = client_2.publish(topic(double{}), val(double{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = client_2.publish(topic((void*)(nullptr)), reinterpret_cast<void*>(0x0), 0);
    ASSERT_EQ(res, 0);
    usleep(250);
    res = client_2.publish(topic(custom_t{}), "Hello, FLECS!", 13, encoding(custom_t{}));
    ASSERT_EQ(res, 0);

    auto lock = std::unique_lock{m};
    cv.wait(lock, [] { return done; });
    done = false;
}

template <typename T>
void flunder_cbk_c_userp(void* client, const FLECS::flunder_variable_t* var, void* userp)
{
    std::fprintf(stderr, "Received topic %s\n", var->topic().data());
    ASSERT_EQ(client, userp);
    ASSERT_EQ(std::string_view{flunder_variable_encoding(var)}, encoding(T{}));
    ASSERT_EQ(std::string_view{flunder_variable_topic(var)}, topic(T{}));
    ASSERT_EQ(std::string_view{flunder_variable_value(var)}, FLECS::stringify(val(T{})));
    ASSERT_EQ(flunder_variable_len(var), FLECS::stringify(val(T{})).length());
}

void flunder_cbk_c(void* /*client*/, const FLECS::flunder_variable_t* var)
{
    std::fprintf(stderr, "Received topic %s\n", var->topic().data());
    if (var->topic() == topic((void*)(nullptr)))
    {
        ASSERT_EQ(var->encoding(), encoding((void*)(nullptr)));
    }
    else if (var->topic() == topic(custom_t{}))
    {
        ASSERT_EQ(var->encoding(), encoding(custom_t{}));
        auto lock_guard = std::lock_guard<std::mutex>{m};
        done = true;
        cv.notify_all();
    }
}

TEST(flunder, c)
{
    using namespace FLECS;

    auto client = flunder_client_new();
    ASSERT_NE(client, nullptr);

    auto res = flunder_connect(client, "172.17.0.1", 7447);
    ASSERT_EQ(res, 0);

    res = flunder_subscribe_userp(client, topic(bool{}), flunder_cbk_c_userp<bool>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic(std::int8_t{}), flunder_cbk_c_userp<std::int8_t>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic(std::int16_t{}), flunder_cbk_c_userp<std::int16_t>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic(std::int32_t{}), flunder_cbk_c_userp<std::int32_t>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic(std::int64_t{}), flunder_cbk_c_userp<std::int64_t>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic(std::uint8_t{}), flunder_cbk_c_userp<std::uint8_t>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic(std::uint16_t{}), flunder_cbk_c_userp<std::uint16_t>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic(std::uint32_t{}), flunder_cbk_c_userp<std::uint32_t>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic(std::uint64_t{}), flunder_cbk_c_userp<std::uint64_t>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic(float{}), flunder_cbk_c_userp<float>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic(double{}), flunder_cbk_c_userp<double>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe_userp(client, topic((const char*)(nullptr)), flunder_cbk_c_userp<const char*>, client);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe(client, topic((void*)(nullptr)), flunder_cbk_c);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe(client, topic(custom_t{}), flunder_cbk_c);
    ASSERT_EQ(res, 0);
    res = flunder_subscribe(client, topic(nullptr), flunder_cbk_c);
    ASSERT_EQ(res, 0);
    res = flunder_unsubscribe(client, topic(nullptr));
    ASSERT_EQ(res, 0);

    res = flunder_publish_bool(client, topic(bool{}), true);
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_int8(client, topic(std::int8_t{}), val(std::int8_t{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_int16(client, topic(std::int16_t{}), val(std::int16_t{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_int32(client, topic(std::int32_t{}), val(std::int32_t{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_int64(client, topic(std::int64_t{}), val(std::int64_t{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_uint8(client, topic(std::uint8_t{}), val(std::uint8_t{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_uint16(client, topic(std::uint16_t{}), val(std::uint16_t{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_uint32(client, topic(std::uint32_t{}), val(std::uint32_t{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_uint64(client, topic(std::uint64_t{}), val(std::uint64_t{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_float(client, topic(float{}), val(float{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_double(client, topic(double{}), val(double{}));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_string(client, topic((const char*)(nullptr)), val((const char*)(nullptr)));
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_raw(client, topic((void*)(nullptr)), nullptr, 0);
    ASSERT_EQ(res, 0);
    usleep(250);
    res = flunder_publish_custom(client, topic(custom_t{}), nullptr, 0, encoding(custom_t{}));
    ASSERT_EQ(res, 0);

    auto lock = std::unique_lock{m};
    cv.wait(lock, [] { return done; });
    done = false;

    res = flunder_disconnect(client);
    ASSERT_EQ(res, 0);
    flunder_client_destroy(client);
}
