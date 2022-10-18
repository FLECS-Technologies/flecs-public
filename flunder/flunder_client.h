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

#ifndef C33E0442_0C18_433F_88A2_9738DDC82A5A
#define C33E0442_0C18_433F_88A2_9738DDC82A5A

/*! @todo */
#ifndef FLECS_EXPORT
#define FLECS_EXPORT
#endif // FLECS_EXPORT

#ifndef __cplusplus
#define FLECS_FLUNDER_HOST "flecs-flunder"
#define FLECS_FLUNDER_PORT 7447
#endif // __cplusplus

#include "flunder_variable.h"

#ifndef __cplusplus
#include <stdbool.h>
#else

#include <functional>
#include <memory>
#include <string>
#include <string_view>
#include <tuple>
#include <vector>

#include "core/global/types/type_traits.h"
#include "util/string/string_utils.h"

namespace FLECS {
namespace Private {
class flunder_client_private_t;
} // namespace Private

/*! DNS name of the default flunder broker */
constexpr const char* FLUNDER_HOST = "flecs-flunder";
/*! Port of the default flunder broker */
constexpr const int FLUNDER_PORT = 7447;

class flunder_client_t
{
public:
    /*! @brief Constructor
     */
    FLECS_EXPORT flunder_client_t();

    /*! @brief Copy constructor (deleted)
     */
    FLECS_EXPORT flunder_client_t(const flunder_client_t&) = delete;

    /*! @brief Move constructor
     */
    FLECS_EXPORT flunder_client_t(flunder_client_t&& other);

    /*! @brief copy-assignment operator (deleted)
     */
    FLECS_EXPORT flunder_client_t& operator=(const flunder_client_t&) = delete;

    /*! @brief move-assignment operator
     */
    FLECS_EXPORT flunder_client_t& operator=(flunder_client_t&& other);

    /*! @brief Destructor
     */
    FLECS_EXPORT ~flunder_client_t();

    FLECS_EXPORT auto connect() //
        -> int;
    FLECS_EXPORT auto connect(std::string_view host, int port) //
        -> int;

    FLECS_EXPORT auto reconnect() //
        -> int;

    FLECS_EXPORT auto disconnect() //
        -> int;

    /* publish typed data to live subscribers */
    /* bool */
    FLECS_EXPORT auto publish(std::string_view topic, bool value) //
        -> int;

    /* integer-types */
    template <typename T>
    FLECS_EXPORT auto publish(std::string_view topic, const T& value) //
        -> std::enable_if_t<std::is_integral_v<T> && !std::is_same_v<T, bool>, int>;

    /* floating-point-types */
    template <typename T>
    FLECS_EXPORT auto publish(std::string_view topic, const T& value) //
        -> std::enable_if_t<std::is_floating_point_v<T>, int>;

    /* string-types */
    template <typename T>
    FLECS_EXPORT auto publish(std::string_view topic, const T& value) //
        -> std::enable_if_t<is_std_string_v<T> || is_std_string_view_v<T>, int>;

    FLECS_EXPORT auto publish(std::string_view topic, const char* value) //
        -> int;

    /* raw data */
    FLECS_EXPORT auto publish(std::string_view topic, const void* data, size_t len) //
        -> int;

    using subscribe_cbk_t = std::function<void(flunder_client_t*, const flunder_variable_t*)>;
    using subscribe_cbk_userp_t = std::function<void(flunder_client_t*, const flunder_variable_t*, const void*)>;

    /* subscribe to live data */
    FLECS_EXPORT auto subscribe(std::string_view topic, subscribe_cbk_t cbk) //
        -> int;
    /* subscribe to live data with userdata */
    FLECS_EXPORT auto subscribe(std::string_view topic, subscribe_cbk_userp_t cbk, const void* userp) //
        -> int;
    /* unsubscribe from live data */
    FLECS_EXPORT auto unsubscribe(std::string_view topic) //
        -> int;

    FLECS_EXPORT auto add_mem_storage(std::string_view name, std::string_view topic) //
        -> int;
    FLECS_EXPORT auto remove_mem_storage(std::string_view name) //
        -> int;

    /* get data from storage */
    FLECS_EXPORT auto get(std::string_view topic) //
        -> std::tuple<int, std::vector<flunder_variable_t> >;
    /* delete data from storage */
    FLECS_EXPORT auto erase(std::string_view topic) //
        -> int;

private:
    FLECS_EXPORT friend auto swap(flunder_client_t& lhs, flunder_client_t& rhs) noexcept //
        -> void;

    FLECS_EXPORT auto publish_bool(std::string_view topic, const std::string& value) //
        -> int;
    FLECS_EXPORT auto publish_int(std::string_view topic, size_t size, bool is_signed, const std::string& value) //
        -> int;
    FLECS_EXPORT auto publish_float(std::string_view topic, size_t size, const std::string& value) //
        -> int;
    FLECS_EXPORT auto publish_string(std::string_view topic, const std::string& value) //
        -> int;
    FLECS_EXPORT auto publish_raw(std::string_view topic, const void* data, size_t len) //
        -> int;

    std::unique_ptr<Private::flunder_client_private_t> _impl;
};

template <typename T>
auto flunder_client_t::publish(std::string_view topic, const T& value) //
    -> std::enable_if_t<std::is_integral_v<T> && !std::is_same_v<T, bool>, int>
{
    return publish_int(topic, sizeof(T), std::is_signed_v<T>, stringify(value));
}

template <typename T>
auto flunder_client_t::publish(std::string_view topic, const T& value) //
    -> std::enable_if_t<std::is_floating_point_v<T>, int>
{
    return publish_float(topic, sizeof(T), stringify(value));
}

template <typename T>
auto flunder_client_t::publish(std::string_view topic, const T& value) //
    -> std::enable_if_t<is_std_string_v<T> || is_std_string_view_v<T>, int>
{
    return publish_string(topic, value);
}

extern "C" {
#endif // __cplusplus

typedef void (*flunder_subscribe_cbk_t)(void*, const flunder_variable_t*);
typedef void (*flunder_subscribe_cbk_userp_t)(void*, const flunder_variable_t*, void*);

FLECS_EXPORT void* flunder_client_new(void);

FLECS_EXPORT void flunder_client_destroy(void* flunder);

FLECS_EXPORT int flunder_connect(void* flunder, const char* host, int port);

FLECS_EXPORT int flunder_reconnect(void* flunder);

FLECS_EXPORT int flunder_disconnect(void* flunder);

FLECS_EXPORT int flunder_subscribe(void* flunder, const char* topic, flunder_subscribe_cbk_t cbk);
FLECS_EXPORT int flunder_subscribe_userp(
    void* flunder, const char* topic, flunder_subscribe_cbk_userp_t cbk, const void* userp);

FLECS_EXPORT int flunder_unsubscribe(void* flunder, const char* topic);

FLECS_EXPORT int flunder_publish_bool(void* flunder, const char* topic, bool value);
FLECS_EXPORT int flunder_publish_int(void* flunder, const char* topic, int value);
FLECS_EXPORT int flunder_publish_float(void* flunder, const char* topic, float value);
FLECS_EXPORT int flunder_publish_double(void* flunder, const char* topic, double value);
FLECS_EXPORT int flunder_publish_string(void* flunder, const char* topic, const char* value);
FLECS_EXPORT int flunder_publish_raw(void* flunder, const char* topic, const void* value, size_t payloadlen);

FLECS_EXPORT int flunder_add_mem_storage(void* flunder, const char* name, const char* topic);
FLECS_EXPORT int flunder_remove_mem_storage(void* flunder, const char* name);

#ifdef __cplusplus
} // extern "C"
} // namespace FLECS
#endif // __cplusplus

#endif // C33E0442_0C18_433F_88A2_9738DDC82A5A
