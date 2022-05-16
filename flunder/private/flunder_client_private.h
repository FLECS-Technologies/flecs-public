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
#include <memory>
#include <tuple>
#include <variant>
#include <vector>

#include "flunder_client.h"
#include "util/json/json.h"

namespace FLECS {
namespace Private {

class flunder_client_private_t
{
public:
    flunder_client_private_t();
    ~flunder_client_private_t();

    FLECS_EXPORT int connect(std::string_view host, int port);

    FLECS_EXPORT int reconnect();

    FLECS_EXPORT int disconnect();

    FLECS_EXPORT int publish(std::string_view path, const std::string& type, const std::string& value);

    FLECS_EXPORT int subscribe(flunder_client_t* client, std::string_view path, flunder_client_t::subscribe_cbk_t cbk);
    FLECS_EXPORT int subscribe(
        flunder_client_t* client, std::string_view path, flunder_client_t::subscribe_cbk_userp_t cbk,
        const void* userp);
    FLECS_EXPORT int unsubscribe(std::string_view path);

    FLECS_EXPORT int add_mem_storage(std::string_view path, std::string_view name);
    FLECS_EXPORT int remove_mem_storage(std::string_view name);

    FLECS_EXPORT auto get(std::string_view path) -> std::tuple<int, std::vector<flunder_variable_t>>;
    FLECS_EXPORT int erase(std::string_view path);

    /*! Function pointer to receive callback */
    using subscribe_cbk_t = std::variant<flunder_client_t::subscribe_cbk_t, flunder_client_t::subscribe_cbk_userp_t>;

    struct subscribe_ctx_t
    {
        flunder_client_t* _client;
        zn_subscriber_t* _sub;
        subscribe_cbk_t _cbk;
        const void* _userp;
    };

private:
    std::vector<std::string> _mem_storages;

    zn_session_t* _zn_session;
    std::map<std::string, subscribe_ctx_t> _subscriptions;
};

} // namespace Private
} // namespace FLECS

#endif // ADF3DE5E_D65D_4E99_8318_021E8B92926E
