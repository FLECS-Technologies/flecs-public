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

#ifndef ADF3DE5E_D65D_4E99_8318_021E8B92926E
#define ADF3DE5E_D65D_4E99_8318_021E8B92926E

#include <zenoh.h>

#include <map>
#include <set>
#include <string>
#include <tuple>
#include <variant>

#include "flunder_client.h"
#include "util/json/json.h"

namespace FLECS {
namespace Private {

class flunder_client_private_t
{
public:
    flunder_client_private_t();
    ~flunder_client_private_t();

    FLECS_EXPORT auto connect(std::string_view host, int port) //
        -> int;

    FLECS_EXPORT auto reconnect() //
        -> int;

    FLECS_EXPORT auto is_connected() const noexcept //
        -> bool;

    FLECS_EXPORT auto disconnect() //
        -> int;

    FLECS_EXPORT auto publish_bool(std::string_view topic, const std::string& value) const //
        -> int;

    FLECS_EXPORT auto publish_int(
        std::string_view topic, size_t size, bool is_signed, const std::string& value) const //
        -> int;

    FLECS_EXPORT auto publish_float(std::string_view topic, size_t size, const std::string& value) const //
        -> int;

    FLECS_EXPORT auto publish_string(std::string_view topic, const std::string& value) const //
        -> int;

    FLECS_EXPORT auto publish_raw(std::string_view topic, const void* payload, size_t payloadlen) const //
        -> int;

    FLECS_EXPORT auto publish_custom(
        std::string_view topic, const void* payload, size_t payloadlen, std::string_view encoding) const //
        -> int;

    FLECS_EXPORT auto subscribe(
        flunder_client_t* client, std::string_view topic, flunder_client_t::subscribe_cbk_t cbk) //
        -> int;

    FLECS_EXPORT auto subscribe(
        flunder_client_t* client,
        std::string_view topic,
        flunder_client_t::subscribe_cbk_userp_t cbk,
        const void* userp) //
        -> int;

    FLECS_EXPORT auto unsubscribe(std::string_view topic) //
        -> int;

    FLECS_EXPORT auto add_mem_storage(std::string topic, std::string_view name) //
        -> int;

    FLECS_EXPORT auto remove_mem_storage(std::string name) //
        -> int;

    FLECS_EXPORT auto get(std::string_view topic) const //
        -> std::tuple<int, std::vector<flunder_variable_t>>;

    FLECS_EXPORT auto erase(std::string_view topic) //
        -> int;

    /*! Function pointer to receive callback */
    using subscribe_cbk_t = std::variant<flunder_client_t::subscribe_cbk_t, flunder_client_t::subscribe_cbk_userp_t>;

    struct subscribe_ctx_t
    {
        flunder_client_t* _client;
        z_owned_subscriber_t _sub;
        subscribe_cbk_t _cbk;
        const void* _userp;
        bool _once;
    };

private:
    FLECS_EXPORT auto publish(std::string_view topic, z_encoding_t encoding, const std::string& value) const //
        -> int;

    FLECS_EXPORT auto subscribe(
        flunder_client_t* client, std::string_view topic, subscribe_cbk_t cbk, const void* userp) //
        -> int;

    std::set<std::string> _mem_storages;

    z_owned_session_t _z_session;
    std::map<std::string, subscribe_ctx_t> _subscriptions;
};

auto to_string(z_encoding_prefix_t prefix, std::string_view suffix) //
    -> std::string;

auto ntp64_to_unix_time(std::uint64_t ntp_time) //
    -> uint64_t;

} // namespace Private
} // namespace FLECS

#endif // ADF3DE5E_D65D_4E99_8318_021E8B92926E
