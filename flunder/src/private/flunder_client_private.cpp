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

#include "private/flunder_client_private.h"

#include <cpr/cpr.h>
#include <json/json.h>

#include <thread>

#include "util/http/status_codes.h"

namespace FLECS {
namespace Private {

/** @todo */
template <class... Ts>
struct overload : Ts...
{
    using Ts::operator()...;
};
template <class... Ts>
overload(Ts...) -> overload<Ts...>;

static void lib_subscribe_callback(const zn_sample_t* sample, const void* arg)
{
    auto key = std::string{sample->key.val, sample->key.len};
    auto var = flunder_data_t{key.c_str(), sample->value.val, sample->value.len};
    const auto* ctx = static_cast<const flunder_client_private_t::subscribe_ctx_t*>(arg);
    std::visit(
        overload{// call callback without userdata
                 [&](flunder_client_t::subscribe_cbk_t cbk) { cbk(ctx->_client, &var); },
                 // call callback with userdata
                 [&](flunder_client_t::subscribe_cbk_userp_t cbk) { cbk(ctx->_client, &var, ctx->_userp); }},
        ctx->_cbk);
}

flunder_client_private_t::flunder_client_private_t()
    : _json_reader{Json::CharReaderBuilder().newCharReader()}
    , _mem_storages{}
    , _zn_session{}
    , _subscriptions{}
{}

flunder_client_private_t::~flunder_client_private_t()
{}

int flunder_client_private_t::connect(std::string_view /*host*/, int /*port*/)
{
    disconnect();

    const auto url = std::string{"http://flecs-flunder:8000"};
    const auto res = cpr::Get(cpr::Url{url});

    auto zn_properties = zn_config_default();
    zn_properties_insert(zn_properties, ZN_CONFIG_PEER_KEY, z_string_make("tcp/flecs-flunder:7447"));
    zn_properties_insert(zn_properties, ZN_CONFIG_MODE_KEY, z_string_make("client"));
    _zn_session = zn_open(zn_properties);

    return (res.status_code == 200 && _zn_session) ? 0 : -1;
}

int flunder_client_private_t::reconnect()
{
    return connect("", 0);
}

int flunder_client_private_t::disconnect()
{
    while (!_subscriptions.empty())
    {
        unsubscribe(_subscriptions.rbegin()->first);
    }
    while (!_mem_storages.empty())
    {
        remove_mem_storage(*_mem_storages.rbegin());
    }
    if (_zn_session)
    {
        zn_close(_zn_session);
        _zn_session = nullptr;
    }
    return 0;
}

int flunder_client_private_t::publish(std::string_view path, const std::string& type, const std::string& value)
{
    const auto url = std::string{"http://flecs-flunder:8000"}.append(path);
    const auto res = cpr::Put(cpr::Url{url}, cpr::Header{{"content-type", type}}, cpr::Body{value});
    return (res.status_code == 200) ? 0 : -1;
}

int flunder_client_private_t::subscribe(
    flunder_client_t* client, std::string_view path, flunder_client_t::subscribe_cbk_t cbk)
{
    if (_subscriptions.count(path.data()) > 0)
    {
        return -1;
    }

    auto ctx = subscribe_ctx_t{client, nullptr, cbk, nullptr};
    auto res = _subscriptions.emplace(path, ctx);
    if (!res.second)
    {
        return -1;
    }

    auto sub = zn_declare_subscriber(
        _zn_session,
        zn_rname(path.data()),
        zn_subinfo_default(),
        lib_subscribe_callback,
        &res.first->second);
    res.first->second._sub = sub;

    if (!sub)
    {
        _subscriptions.erase(res.first);
        return -1;
    }

    return 0;
}

int flunder_client_private_t::subscribe(
    flunder_client_t* client, std::string_view path, flunder_client_t::subscribe_cbk_userp_t cbk, const void* userp)
{
    if (_subscriptions.count(path.data()) > 0)
    {
        return -1;
    }

    auto ctx = subscribe_ctx_t{client, nullptr, cbk, userp};
    auto res = _subscriptions.emplace(path, ctx);
    if (!res.second)
    {
        return -1;
    }

    auto sub = zn_declare_subscriber(
        _zn_session,
        zn_rname(path.data()),
        zn_subinfo_default(),
        lib_subscribe_callback,
        &res.first->second);
    res.first->second._sub = sub;

    if (!sub)
    {
        _subscriptions.erase(res.first);
        return -1;
    }

    return 0;
}

int flunder_client_private_t::unsubscribe(std::string_view path)
{
    auto it = _subscriptions.find(path.data());
    if (it == _subscriptions.cend())
    {
        return -1;
    }

    zn_undeclare_subscriber(it->second._sub);
    _subscriptions.erase(it);

    return 0;
}

int flunder_client_private_t::add_mem_storage(std::string_view name, std::string_view path)
{
    auto url = cpr::Url{std::string{"http://flecs-flunder:8000"}
                            .append("/@/router/local/plugin/storages/backend/memory/storage/")
                            .append(name)};
    auto body = cpr::Body{std::string{"path_expr="}.append(path)};
    const auto res = cpr::Put(url, cpr::Header{{"content-type", "application/properties"}}, body);

    if (res.status_code != 200)
    {
        return -1;
    }

    _mem_storages.emplace_back(name);

    return 0;
}

int flunder_client_private_t::remove_mem_storage(std::string_view name)
{
    auto url = cpr::Url{std::string{"http://flecs-flunder:8000"}
                            .append("/@/router/local/plugin/storages/backend/memory/storage/")
                            .append(name)};
    const auto res = cpr::Delete(url);

    if (res.status_code != 200)
    {
        return -1;
    }

    _mem_storages.erase(
        std::remove_if(_mem_storages.begin(), _mem_storages.end(), [&](const std::string& str) { return str == name; }),
        _mem_storages.end());

    return 0;
}

auto flunder_client_private_t::get(std::string_view path) -> std::tuple<int, std::vector<flunder_variable_t>>
{
    auto vars = std::vector<flunder_variable_t>{};

    const auto url = std::string{"http://flecs-flunder:8000"}.append(path);
    const auto res = cpr::Get(cpr::Url{url});

    if (res.status_code != static_cast<long>(http_status_e::Ok))
    {
        return {-1, vars};
    }

    decltype(auto) str = res.text;
    auto json = Json::Value{};
    if (!_json_reader->parse(res.text.c_str(), res.text.c_str() + res.text.length(), &json, nullptr))
    {
        return {-1, vars};
    }

    for (decltype(auto) it = json.begin(); it != json.end(); ++it)
    {
        vars.emplace_back(flunder_variable_t{
            (*it)["key"].as<std::string>().c_str(),
            (*it)["value"].as<std::string>().c_str(),
            (*it)["encoding"].as<std::string>().c_str(),
            (*it)["time"].as<std::string>().c_str()});
    }

    return {0, vars};
}

int flunder_client_private_t::erase(std::string_view path)
{
    const auto url = std::string{"http://flecs-flunder:8000"}.append(path);
    const auto res = cpr::Delete(cpr::Url{url});
    return (res.status_code == 200) ? 0 : -1;
}

} // namespace Private
} // namespace FLECS
