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

#include <thread>

#include "util/cxx20/string.h"

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

static auto lib_subscribe_callback(const z_sample_t* sample, void* arg) //
    -> void
{
    const auto* ctx = static_cast<const flunder_client_private_t::subscribe_ctx_t*>(arg);
    if (!ctx->_once)
    {
        return;
    }

    const auto keyexpr = z_keyexpr_to_string(sample->keyexpr);
    const auto var = flunder_variable_t{
        std::string{"/"} + std::string{keyexpr},
        std::string{reinterpret_cast<const char*>(sample->payload.start), sample->payload.len},
        to_string(
            sample->encoding.prefix,
            std::string_view{
                reinterpret_cast<const char*>(sample->encoding.suffix.start),
                sample->encoding.suffix.len}),
        stringify(ntp64_to_unix_time(sample->timestamp.time))};
    free(keyexpr);

    std::visit(
        overload{// call callback without userdata
                 [&](flunder_client_t::subscribe_cbk_t cbk) { cbk(ctx->_client, &var); },
                 // call callback with userdata
                 [&](flunder_client_t::subscribe_cbk_userp_t cbk) { cbk(ctx->_client, &var, ctx->_userp); }},
        ctx->_cbk);
}

auto to_string(z_encoding_prefix_t prefix, std::string_view suffix) //
    -> std::string
{
    using std::operator""s;

    const auto strings = std::map<z_encoding_prefix_t, std::string_view>{
        {Z_ENCODING_PREFIX_EMPTY, ""},
        {Z_ENCODING_PREFIX_APP_OCTET_STREAM, "application/octet-stream"},
        {Z_ENCODING_PREFIX_APP_CUSTOM, "application/custom"},
        {Z_ENCODING_PREFIX_TEXT_PLAIN, "text/plain"},
        {Z_ENCODING_PREFIX_APP_PROPERTIES, "application/properties"},
        {Z_ENCODING_PREFIX_APP_JSON, "application/json"},
        {Z_ENCODING_PREFIX_APP_SQL, "application/sql"},
        {Z_ENCODING_PREFIX_APP_INTEGER, "application/integer"},
        {Z_ENCODING_PREFIX_APP_FLOAT, "application/float"},
        {Z_ENCODING_PREFIX_APP_XML, "application/xml"},
        {Z_ENCODING_PREFIX_APP_XHTML_XML, "application/xhtml+xml"},
        {Z_ENCODING_PREFIX_APP_X_WWW_FORM_URLENCODED, "application/x-www-form-urlencoded"},
        {Z_ENCODING_PREFIX_TEXT_JSON, "text/json"},
        {Z_ENCODING_PREFIX_TEXT_HTML, "text/html"},
        {Z_ENCODING_PREFIX_TEXT_XML, "text/xml"},
        {Z_ENCODING_PREFIX_TEXT_CSS, "text/css"},
        {Z_ENCODING_PREFIX_TEXT_CSV, "text/csv"},
        {Z_ENCODING_PREFIX_TEXT_JAVASCRIPT, "text/javascript"},
        {Z_ENCODING_PREFIX_IMAGE_JPEG, "image/jpeg"},
        {Z_ENCODING_PREFIX_IMAGE_PNG, "image/png"},
        {Z_ENCODING_PREFIX_IMAGE_GIF, "image/gif"},
    };

    const auto it = strings.find(prefix);
    return (it != strings.cend()) ? std::string{it->second.data()} + std::string{suffix} : std::string{suffix};
}

flunder_client_private_t::flunder_client_private_t()
    : _mem_storages{}
    , _z_session{}
    , _subscriptions{}
{}

flunder_client_private_t::~flunder_client_private_t()
{}

auto flunder_client_private_t::connect(std::string_view /*host*/, int /*port*/) //
    -> int
{
    auto config = z_config_default();
    zc_config_insert_json(z_config_loan(&config), Z_CONFIG_CONNECT_KEY, "tcp/flecs-flunder:7447");
    zc_config_insert_json(z_config_loan(&config), Z_CONFIG_MODE_KEY, "client");
    zc_config_insert_json(z_config_loan(&config), "timestamping/enabled", "true");

    _z_session = z_open(z_move(config));

    return z_session_check(&_z_session) ? 0 : -1;
}

auto flunder_client_private_t::reconnect() //
    -> int
{
    disconnect();
    return connect("", 0);
}

auto flunder_client_private_t::disconnect() //
    -> int
{
    while (!_subscriptions.empty())
    {
        unsubscribe(_subscriptions.rbegin()->first);
    }
    while (!_mem_storages.empty())
    {
        remove_mem_storage(*_mem_storages.rbegin());
    }
    if (z_session_check(&_z_session))
    {
        z_close(z_move(_z_session));
    }
    return 0;
}

auto flunder_client_private_t::publish_bool(std::string_view topic, const std::string& value) //
    -> int
{
    return publish(topic, z_encoding(Z_ENCODING_PREFIX_APP_CUSTOM, "bool"), value);
}

auto flunder_client_private_t::publish_int(
    std::string_view topic, size_t size, bool is_signed, const std::string& value) //
    -> int
{
    using std::operator""s;

    auto suffix = is_signed ? "+s"s : "+u"s;
    suffix += stringify(size * 8);

    return publish(topic, z_encoding(Z_ENCODING_PREFIX_APP_INTEGER, suffix.c_str()), value);
}

auto flunder_client_private_t::publish_float(std::string_view topic, size_t size, const std::string& value) //
    -> int
{
    const auto size_str = stringify("+", size * 8);
    return publish(topic, z_encoding(Z_ENCODING_PREFIX_APP_FLOAT, size_str.c_str()), value);
}

auto flunder_client_private_t::publish_string(std::string_view topic, const std::string& value) //
    -> int
{
    return publish(topic, z_encoding(Z_ENCODING_PREFIX_TEXT_PLAIN, nullptr), value);
}

auto flunder_client_private_t::publish_raw(std::string_view topic, const void* payload, size_t payloadlen) //
    -> int
{
    return publish(
        topic,
        z_encoding(Z_ENCODING_PREFIX_APP_OCTET_STREAM, nullptr),
        std::string{reinterpret_cast<const char*>(payload), payloadlen});
}

auto flunder_client_private_t::publish(std::string_view topic, z_encoding_t encoding, const std::string& value) //
    -> int
{
    auto options = z_put_options_default();
    options.encoding = encoding;
    options.congestion_control = z_congestion_control_t::Z_CONGESTION_CONTROL_BLOCK;

    const auto keyexpr = cxx20::starts_with(topic, '/') ? topic.data() + 1 : topic.data();

    const auto res = z_put(
        z_session_loan(&_z_session),
        z_keyexpr(keyexpr),
        reinterpret_cast<const uint8_t*>(value.data()),
        value.size(),
        &options);

    return (res == 1) ? 0 : -1;
}

auto flunder_client_private_t::subscribe(
    flunder_client_t* client,
    std::string_view topic,
    flunder_client_t::subscribe_cbk_t cbk) //
    -> int
{
    return subscribe(client, topic, subscribe_cbk_t{cbk}, nullptr);
}

auto flunder_client_private_t::subscribe(
    flunder_client_t* client,
    std::string_view topic,
    flunder_client_t::subscribe_cbk_userp_t cbk,
    const void* userp) //
    -> int
{
    return subscribe(client, topic, subscribe_cbk_t{cbk}, userp);
}

auto flunder_client_private_t::subscribe(
    flunder_client_t* client,
    std::string_view topic,
    subscribe_cbk_t cbk,
    const void* userp) //
    -> int
{
    const auto keyexpr = cxx20::starts_with(topic, '/') ? topic.data() + 1 : topic.data();

    if (_subscriptions.count(keyexpr) > 0)
    {
        return -1;
    }

    auto res = _subscriptions.emplace(keyexpr, subscribe_ctx_t{client, {}, cbk, userp, false});
    if (!res.second)
    {
        return -1;
    }
    auto& ctx = res.first->second;

    auto options = z_subscriber_options_default();
    options.reliability = Z_RELIABILITY_RELIABLE;

    auto closure = z_owned_closure_sample_t{.context = &ctx, .call = lib_subscribe_callback, .drop = nullptr};
    ctx._sub = z_declare_subscriber(z_session_loan(&_z_session), z_keyexpr(keyexpr), z_move(closure), &options);

    if (!z_subscriber_check(&ctx._sub))
    {
        _subscriptions.erase(res.first);
        return -1;
    }

    const auto [unused, vars] = get(keyexpr);
    for (const auto& var : vars)
    {
        std::visit(
            overload{// call callback without userdata
                     [&](flunder_client_t::subscribe_cbk_t cbk) { cbk(ctx._client, &var); },
                     // call callback with userdata
                     [&](flunder_client_t::subscribe_cbk_userp_t cbk) { cbk(ctx._client, &var, ctx._userp); }},
            ctx._cbk);
    }
    ctx._once = true;

    return 0;
}

auto flunder_client_private_t::unsubscribe(std::string_view topic) //
    -> int
{
    auto it = _subscriptions.find(topic.data());
    if (it == _subscriptions.cend())
    {
        return -1;
    }

    z_undeclare_subscriber(z_move(it->second._sub));
    _subscriptions.erase(it);

    return 0;
}

auto flunder_client_private_t::add_mem_storage(std::string name, std::string_view topic) //
    -> int
{
    if (_mem_storages.count(name))
    {
        return -1;
    }

    const auto keyexpr = cxx20::starts_with(topic, '/') ? topic.data() + 1 : topic.data();

    auto url = cpr::Url{std::string{"http://flecs-flunder:8000"}
                            .append("/@/router/local/config/plugins/storage_manager/storages/")
                            .append(name)};

    const auto req_json = json_t{{"key_expr", keyexpr}, {"volume", "memory"}};
    const auto res = cpr::Put(url, cpr::Header{{"content-type", "application/json"}}, cpr::Body{req_json.dump()});

    if (res.status_code != 200)
    {
        return -1;
    }

    _mem_storages.insert(std::move(name));

    return 0;
}

auto flunder_client_private_t::remove_mem_storage(std::string name) //
    -> int
{
    if (!_mem_storages.count(name))
    {
        return -1;
    }

    auto url = cpr::Url{std::string{"http://flecs-flunder:8000"}
                            .append("/@/router/local/config/plugins/storage_manager/storages/")
                            .append(name)};
    const auto res = cpr::Delete(url);

    if (res.status_code != 200)
    {
        return -1;
    }

    _mem_storages.erase(name);

    return 0;
}

auto flunder_client_private_t::get(std::string_view topic) //
    -> std::tuple<int, std::vector<flunder_variable_t>>
{
    auto vars = std::vector<flunder_variable_t>{};

    if (!z_session_check(&_z_session))
    {
        return {-1, vars};
    }

    auto reply_channel = zc_reply_fifo_new(64);
    auto options = z_get_options_default();
    options.target = Z_QUERY_TARGET_ALL;

    auto keyexpr = z_keyexpr(topic.data());
    if (!z_keyexpr_is_initialized(&keyexpr))
    {
        return {-1, vars};
    }

    z_get(z_session_loan(&_z_session), keyexpr, "", z_move(reply_channel.send), &options);

    z_owned_reply_t reply = z_reply_null();
    for (z_reply_channel_closure_call(&reply_channel.recv, z_move(reply)); z_reply_check(&reply);
         z_reply_channel_closure_call(&reply_channel.recv, z_move(reply)))
    {
        if (z_reply_is_ok(&reply))
        {
            const auto sample = z_reply_ok(&reply);
            char* const keyexpr = z_keyexpr_to_string(sample.keyexpr);
            auto keystr = std::string{"/"} + std::string{keyexpr};
            free(keyexpr);
            if (cxx20::starts_with(keystr, "/@"))
            {
                continue;
            }

            vars.emplace_back(
                std::move(keystr),
                std::string(reinterpret_cast<const char*>(sample.payload.start), sample.payload.len),
                to_string(
                    sample.encoding.prefix,
                    std::string_view{
                        reinterpret_cast<const char*>(sample.encoding.suffix.start),
                        sample.encoding.suffix.len}),
                stringify(ntp64_to_unix_time(sample.timestamp.time)));
        }
    }

    z_reply_drop(z_move(reply));
    z_reply_channel_drop(z_move(reply_channel));

    return {0, vars};
}

auto flunder_client_private_t::erase(std::string_view topic) //
    -> int
{
    const auto keyexpr = cxx20::starts_with(topic, '/') ? topic.data() + 1 : topic.data();

    auto options = z_delete_options_default();
    const auto res = z_delete(z_session_loan(&_z_session), z_keyexpr(keyexpr), &options);

    return (res == 1) ? 0 : -1;
}

auto ntp64_to_unix_time(std::uint64_t ntp_time) //
    -> uint64_t
{
    //           ntp 64-bit time
    // byte    7        6        5        4
    //  -------- -------- -------- --------
    // |             seconds               |
    //  -------- -------- -------- --------
    //
    // byte    3        2        1        0
    //  -------- -------- -------- --------
    // |            fractions              |
    //  -------- -------- -------- --------
    //
    // 1 fraction == 1/2^32 seconds (approx 232 ps)

    const auto seconds = ntp_time >> 32;
    const auto fractions = static_cast<double>(ntp_time & 0xffffffff);
    const auto unix_time =
        static_cast<uint64_t>((seconds + (fractions / std::numeric_limits<std::uint32_t>::max())) * 1'000'000'000);

    return unix_time;
}

} // namespace Private
} // namespace FLECS
