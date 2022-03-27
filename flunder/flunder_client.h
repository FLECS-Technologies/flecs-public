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

#include <memory>
#include <string>
#include <string_view>
#include <tuple>
#include <vector>

#include "core/global/types/type_traits.h"

namespace FLECS {
namespace Private {
class flunder_client_private_t;
} // namespace Private

/*! DNS name of the default FLECS MQTT broker */
constexpr const char* FLUNDER_HOST = "flecs-flunder";
/*! Port of the default FLECS MQTT broker */
constexpr const int FLUNDER_PORT = 8000;

struct flunder_data_t
{
    std::string path;
    void* data;
};

struct flunder_variable_t
{
    std::string key;
    std::string value;
    std::string encoding;
    std::string timestamp;
};

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

    FLECS_EXPORT int connect();
    FLECS_EXPORT int connect(std::string_view host, int port);

    FLECS_EXPORT int reconnect();

    FLECS_EXPORT int disconnect();

    /* publish typed data to live subscribers */
    /* integer-types */
    template <typename T>
    FLECS_EXPORT auto publish(std::string_view path, const T& value) -> std::enable_if_t<std::is_integral_v<T>, int>;

    /* floating-point-types */
    template <typename T>
    FLECS_EXPORT auto publish(std::string_view path, const T& value)
        -> std::enable_if_t<std::is_floating_point_v<T>, int>;

    /* string-types */
    template <typename T>
    FLECS_EXPORT auto publish(std::string_view path, const T& value)
        -> std::enable_if_t<is_std_string_v<T> || is_std_string_view_v<T>, int>;

    FLECS_EXPORT int publish(std::string_view path, const char* value);

    /* raw data */
    FLECS_EXPORT int publish(std::string_view path, const void* data, size_t len);

    // using subscribe_callback_t = void (*)(const flunder_data_t*, const void*);
    /* subscribe to live data */
    // FLECS_EXPORT int subscribe(std::string_view path, const subscribe_callback_t& cbk);
    /* unsubscribe from live data */
    // FLECS_EXPORT int unsubscribe(std::string_view path);

    FLECS_EXPORT int add_mem_storage(std::string_view name, std::string_view path);
    FLECS_EXPORT int remove_mem_storage(std::string_view name);

    /* get data from storage */
    FLECS_EXPORT auto get(std::string_view path) -> std::tuple<int, std::vector<flunder_variable_t>>;
    /* delete data from storage */
    FLECS_EXPORT int erase(std::string_view path);

private:
    friend FLECS_EXPORT void swap(flunder_client_t& lhs, flunder_client_t& rhs) noexcept;

    FLECS_EXPORT int publish_int(std::string_view path, const std::string& value);
    FLECS_EXPORT int publish_float(std::string_view path, const std::string& value);
    FLECS_EXPORT int publish_string(std::string_view path, const std::string& value);
    FLECS_EXPORT int publish_raw(std::string_view path, const std::string& value);

    std::unique_ptr<Private::flunder_client_private_t> _impl;
};

template <typename T>
auto flunder_client_t::publish(std::string_view path, const T& value) -> std::enable_if_t<std::is_integral_v<T>, int>
{
    return publish_int(path, stringify(value));
}

template <typename T>
auto flunder_client_t::publish(std::string_view path, const T& value)
    -> std::enable_if_t<std::is_floating_point_v<T>, int>
{
    return publish_float(path, stringify(value));
}

template <typename T>
auto flunder_client_t::publish(std::string_view path, const T& value)
    -> std::enable_if_t<is_std_string_v<T> || is_std_string_view_v<T>, int>
{
    return publish_string(path, value);
}

} // namespace FLECS

#endif // C33E0442_0C18_433F_88A2_9738DDC82A5A
