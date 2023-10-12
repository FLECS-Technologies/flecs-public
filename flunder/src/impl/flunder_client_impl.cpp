// Copyright 2021-2023 FLECS Technologies GmbH
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

#include "impl/flunder_client_impl.h"

#include <cpr/cpr.h>

#include <thread>

#include "util/cxx20/string.h"

namespace FLECS {
namespace impl {

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
    const auto* ctx = static_cast<const flunder_client_t::subscribe_ctx_t*>(arg);
    if (!ctx->_once) {
        return;
    }

    auto keyexpr = z_keyexpr_to_string(sample->keyexpr);
    const auto var = flunder_variable_t{
        std::string{"/"} + std::string{keyexpr._cstr},
        std::string{reinterpret_cast<const char*>(sample->payload.start), sample->payload.len},
        to_string(
            sample->encoding.prefix,
            std::string_view{
                reinterpret_cast<const char*>(sample->encoding.suffix.start),
                sample->encoding.suffix.len}),
        stringify(ntp64_to_unix_time(sample->timestamp.time))};
    z_str_drop(z_move(keyexpr));

    std::visit(
        overload{// call callback without userdata
                 [&](flunder_client_t::subscribe_cbk_t cbk) { cbk(ctx->_client, &var); },
                 // call callback with userdata
                 [&](flunder_client_t::subscribe_cbk_userp_t cbk) {
                     cbk(ctx->_client, &var, ctx->_userp);
                 }},
        ctx->_cbk);
}

auto to_string(z_encoding_prefix_t prefix, std::string_view suffix) //
    -> std::string
{
    const auto strings = std::map<z_encoding_prefix_t, std::string_view>{
        {Z_ENCODING_PREFIX_EMPTY, ""},
        {Z_ENCODING_PREFIX_APP_OCTET_STREAM, "application/octet-stream"},
        {Z_ENCODING_PREFIX_APP_CUSTOM, "application/"},
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
    return (it != strings.cend()) ? std::string{it->second.data()} + std::string{suffix}
                                  : std::string{suffix};
}

auto encoding_from_string(std::string_view encoding) //
    -> std::tuple<z_encoding_prefix_t, std::string_view>
{
    const auto encodings = std::map<std::string_view, z_encoding_prefix_t>{
        {"", Z_ENCODING_PREFIX_EMPTY},
        {"application/octet-stream", Z_ENCODING_PREFIX_APP_OCTET_STREAM},
        {"application/", Z_ENCODING_PREFIX_APP_CUSTOM},
        {"text/plain", Z_ENCODING_PREFIX_TEXT_PLAIN},
        {"application/properties", Z_ENCODING_PREFIX_APP_PROPERTIES},
        {"application/json", Z_ENCODING_PREFIX_APP_JSON},
        {"application/sql", Z_ENCODING_PREFIX_APP_SQL},
        {"application/integer", Z_ENCODING_PREFIX_APP_INTEGER},
        {"application/float", Z_ENCODING_PREFIX_APP_FLOAT},
        {"application/xml", Z_ENCODING_PREFIX_APP_XML},
        {"application/xhtml+xml", Z_ENCODING_PREFIX_APP_XHTML_XML},
        {"application/x-www-form-urlencoded", Z_ENCODING_PREFIX_APP_X_WWW_FORM_URLENCODED},
        {"text/json", Z_ENCODING_PREFIX_TEXT_JSON},
        {"text/html", Z_ENCODING_PREFIX_TEXT_HTML},
        {"text/xml", Z_ENCODING_PREFIX_TEXT_XML},
        {"text/css", Z_ENCODING_PREFIX_TEXT_CSS},
        {"text/csv", Z_ENCODING_PREFIX_TEXT_CSV},
        {"text/javascript", Z_ENCODING_PREFIX_TEXT_JAVASCRIPT},
        {"image/jpeg", Z_ENCODING_PREFIX_IMAGE_JPEG},
        {"image/png", Z_ENCODING_PREFIX_IMAGE_PNG},
        {"image/gif", Z_ENCODING_PREFIX_IMAGE_GIF},
    };

    const auto it = encodings.find(encoding);

    if (it != encodings.cend()) {
        return {it->second, std::string_view{}};
    }

    for (const auto& it : encodings) {
        if (!it.first.empty() && cxx20::starts_with(encoding, it.first)) {
            return {it.second, encoding.substr(it.first.length())};
        }
    }

    return {Z_ENCODING_PREFIX_EMPTY, encoding};
}

flunder_client_t::flunder_client_t()
    : _mem_storages{}
    , _z_session{}
    , _subscriptions{}
{}

flunder_client_t::~flunder_client_t()
{}

auto flunder_client_t::connect(std::string_view host, int port) //
    -> int
{
    _host = host;
    _port = port;

    const auto len = std::snprintf(nullptr, 0, "[\"tcp/%s:%d\"]", host.data(), port);
    auto remote = std::string(len, '0');
    std::snprintf(remote.data(), remote.length() + 1, "[\"tcp/%s:%d\"]", host.data(), port);

    auto config = z_config_default();
    zc_config_insert_json(z_config_loan(&config), Z_CONFIG_CONNECT_KEY, remote.c_str());
    zc_config_insert_json(z_config_loan(&config), Z_CONFIG_MODE_KEY, R"#("client")#");
    zc_config_insert_json(z_config_loan(&config), Z_CONFIG_MULTICAST_SCOUTING_KEY, "false");
    zc_config_insert_json(z_config_loan(&config), "timestamping/enabled", "true");

    _z_session = z_open(z_move(config));

    return is_connected() ? 0 : -1;
}

auto flunder_client_t::is_connected() const noexcept //
    -> bool
{
    const auto invalid_session = z_owned_session_t{};

    const auto session_initialized = !std::equal(
        reinterpret_cast<const char*>(&_z_session),
        reinterpret_cast<const char*>(&_z_session) + sizeof(decltype(_z_session)),
        reinterpret_cast<const char*>(&invalid_session));

    return session_initialized && z_session_check(&_z_session);
}

auto flunder_client_t::reconnect() //
    -> int
{
    const auto host = _host;
    const auto port = _port;

    disconnect();
    return connect(host, port);
}

auto flunder_client_t::disconnect() //
    -> int
{
    while (!_subscriptions.empty()) {
        unsubscribe(_subscriptions.rbegin()->first);
    }
    while (!_mem_storages.empty()) {
        remove_mem_storage(*_mem_storages.rbegin());
    }
    if (is_connected()) {
        z_close(z_move(_z_session));
    }
    _host.clear();
    _port = 0;

    return 0;
}

auto flunder_client_t::publish_bool(std::string_view topic, const std::string& value) const //
    -> int
{
    return publish(topic, z_encoding(Z_ENCODING_PREFIX_APP_CUSTOM, "bool"), value);
}

auto flunder_client_t::publish_int(
    std::string_view topic, size_t size, bool is_signed, const std::string& value) const //
    -> int
{
    using std::operator""s;

    auto suffix = is_signed ? "+s"s : "+u"s;
    suffix += stringify(size * 8);

    return publish(topic, z_encoding(Z_ENCODING_PREFIX_APP_INTEGER, suffix.c_str()), value);
}

auto flunder_client_t::publish_float(
    std::string_view topic, size_t size, const std::string& value) const //
    -> int
{
    const auto size_str = stringify("+", size * 8);
    return publish(topic, z_encoding(Z_ENCODING_PREFIX_APP_FLOAT, size_str.c_str()), value);
}

auto flunder_client_t::publish_string(std::string_view topic, const std::string& value) const //
    -> int
{
    return publish(topic, z_encoding(Z_ENCODING_PREFIX_TEXT_PLAIN, nullptr), value);
}

auto flunder_client_t::publish_raw(
    std::string_view topic, const void* payload, size_t payloadlen) const //
    -> int
{
    return publish(
        topic,
        z_encoding(Z_ENCODING_PREFIX_APP_OCTET_STREAM, nullptr),
        std::string{reinterpret_cast<const char*>(payload), payloadlen});
}

auto flunder_client_t::publish_custom(
    std::string_view topic,
    const void* payload,
    size_t payloadlen,
    std::string_view encoding) const //
    -> int
{
    const auto [prefix, suffix] = encoding_from_string(encoding);
    return publish(
        topic,
        z_encoding(prefix, suffix.data()),
        std::string{reinterpret_cast<const char*>(payload), payloadlen});
}

auto flunder_client_t::publish(
    std::string_view topic, z_encoding_t encoding, const std::string& value) const //
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

    return (res == 0) ? 0 : -1;
}

auto flunder_client_t::subscribe(
    FLECS::flunder_client_t* client,
    std::string_view topic,
    flunder_client_t::subscribe_cbk_t cbk) //
    -> int
{
    return subscribe(client, topic, subscribe_cbk_var_t{cbk}, nullptr);
}

auto flunder_client_t::subscribe(
    FLECS::flunder_client_t* client,
    std::string_view topic,
    flunder_client_t::subscribe_cbk_userp_t cbk,
    const void* userp) //
    -> int
{
    return subscribe(client, topic, subscribe_cbk_var_t{cbk}, userp);
}

auto flunder_client_t::subscribe(
    FLECS::flunder_client_t* client,
    std::string_view topic,
    subscribe_cbk_var_t cbk,
    const void* userp) //
    -> int
{
    const auto keyexpr = cxx20::starts_with(topic, '/') ? topic.data() + 1 : topic.data();

    if (_subscriptions.count(keyexpr) > 0) {
        return -1;
    }

    auto res = _subscriptions.emplace(keyexpr, subscribe_ctx_t{client, {}, cbk, userp, false});
    if (!res.second) {
        return -1;
    }
    auto& ctx = res.first->second;

    auto options = z_subscriber_options_default();
    options.reliability = Z_RELIABILITY_RELIABLE;

    auto closure =
        z_owned_closure_sample_t{.context = &ctx, .call = lib_subscribe_callback, .drop = nullptr};
    ctx._sub = z_declare_subscriber(
        z_session_loan(&_z_session),
        z_keyexpr(keyexpr),
        z_move(closure),
        &options);

    if (!z_subscriber_check(&ctx._sub)) {
        _subscriptions.erase(res.first);
        return -1;
    }

    const auto [unused, vars] = get(keyexpr);
    for (const auto& var : vars) {
        std::visit(
            overload{// call callback without userdata
                     [&](flunder_client_t::subscribe_cbk_t cbk) { cbk(ctx._client, &var); },
                     // call callback with userdata
                     [&](flunder_client_t::subscribe_cbk_userp_t cbk) {
                         cbk(ctx._client, &var, ctx._userp);
                     }},
            ctx._cbk);
    }
    ctx._once = true;

    return 0;
}

auto flunder_client_t::unsubscribe(std::string_view topic) //
    -> int
{
    const auto keyexpr = cxx20::starts_with(topic, '/') ? topic.data() + 1 : topic.data();

    auto it = _subscriptions.find(keyexpr);
    if (it == _subscriptions.cend()) {
        return -1;
    }

    z_undeclare_subscriber(z_move(it->second._sub));
    _subscriptions.erase(it);

    return 0;
}

auto flunder_client_t::add_mem_storage(std::string name, std::string_view topic) //
    -> int
{
    if (_mem_storages.count(name)) {
        return -1;
    }

    const auto keyexpr = cxx20::starts_with(topic, '/') ? topic.data() + 1 : topic.data();

    auto url = cpr::Url{std::string{"http://"}
                            .append(_host)
                            .append(":8000")
                            .append("/@/router/local/config/plugins/storage_manager/storages/")
                            .append(name)};

    const auto req_json = json_t{{"key_expr", keyexpr}, {"volume", "memory"}};
    const auto res = cpr::Put(
        url,
        cpr::Header{{"content-type", "application/json"}},
        cpr::Body{req_json.dump()});

    if (res.status_code != 200) {
        return -1;
    }

    _mem_storages.insert(std::move(name));

    return 0;
}

auto flunder_client_t::remove_mem_storage(std::string name) //
    -> int
{
    if (!_mem_storages.count(name)) {
        return -1;
    }

    auto url = cpr::Url{std::string{"http://"}
                            .append(_host)
                            .append(":8000")
                            .append("/@/router/local/config/plugins/storage_manager/storages/")
                            .append(name)};
    const auto res = cpr::Delete(url);

    if (res.status_code != 200) {
        return -1;
    }

    _mem_storages.erase(name);

    return 0;
}

auto flunder_client_t::get(std::string_view topic) const //
    -> std::tuple<int, std::vector<flunder_variable_t>>
{
    auto vars = std::vector<flunder_variable_t>{};

    if (!is_connected()) {
        return {-1, vars};
    }

    auto reply_channel = zc_reply_fifo_new(64);
    auto options = z_get_options_default();
    options.target = Z_QUERY_TARGET_ALL;

    auto keyexpr = z_keyexpr(cxx20::starts_with(topic, '/') ? topic.data() + 1 : topic.data());
    if (!z_keyexpr_is_initialized(&keyexpr)) {
        return {-1, vars};
    }

    z_get(z_session_loan(&_z_session), keyexpr, "", z_move(reply_channel.send), &options);

    z_owned_reply_t reply = z_reply_null();
    for (z_reply_channel_closure_call(&reply_channel.recv, &reply); z_reply_check(&reply);
         z_reply_channel_closure_call(&reply_channel.recv, &reply)) {
        if (z_reply_is_ok(&reply)) {
            const auto sample = z_reply_ok(&reply);
            auto keyexpr = z_keyexpr_to_string(sample.keyexpr);
            auto keystr = std::string{"/"} + std::string{keyexpr._cstr};
            z_str_drop(z_move(keyexpr));
            if (cxx20::starts_with(keystr, "/@")) {
                continue;
            }

            vars.emplace_back(
                std::move(keystr),
                std::string(
                    reinterpret_cast<const char*>(sample.payload.start),
                    sample.payload.len),
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

auto flunder_client_t::erase(std::string_view topic) //
    -> int
{
    const auto keyexpr = cxx20::starts_with(topic, '/') ? topic.data() + 1 : topic.data();

    auto options = z_delete_options_default();
    const auto res = z_delete(z_session_loan(&_z_session), z_keyexpr(keyexpr), &options);

    return (res == 0) ? 0 : -1;
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
    const auto unix_time = static_cast<uint64_t>(
        (seconds + (fractions / std::numeric_limits<std::uint32_t>::max())) * 1'000'000'000);

    return unix_time;
}

} // namespace impl
} // namespace FLECS
