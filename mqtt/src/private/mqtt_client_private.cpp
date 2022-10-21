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

#include "private/mqtt_client_private.h"

#include <limits.h>
#include <mosquitto.h>
#include <unistd.h>

#include <cstring>
#include <random>

#include "mqtt_errors.h"

namespace FLECS {
namespace Private {

__attribute__((constructor)) void mqtt_client_private_ctor()
{
    mosquitto_lib_init();
}

__attribute__((destructor)) void mqtt_client_private_dtor()
{
    mosquitto_lib_cleanup();
}

mqtt_client_private_t::mqtt_client_private_t()
    : _mosq{}
{
    // ClientId: hostname-random
    // hostname       -       random
    // ^ HOSTNAME_MAX ^ + 1 + ^ 8
    auto client_id = std::string{};
    client_id.resize(HOST_NAME_MAX + 1 + 8);
    gethostname(client_id.data(), HOST_NAME_MAX);

    auto seed = std::random_device{};
    auto generator = std::mt19937{seed()};
    auto distribution = std::uniform_int_distribution{
        std::numeric_limits<std::uint32_t>::min(),
        std::numeric_limits<std::uint32_t>::max()};

    auto id = distribution(generator);
    const auto offset = std::strlen(client_id.c_str());
    std::snprintf(client_id.data() + offset, client_id.length() + 1 - offset, "-%.8x", id);

    _mosq = mosquitto_new(client_id.c_str(), true, this);

    mosquitto_message_callback_set(_mosq, &mqtt_client_private_t::lib_receive_callback);
    mosquitto_connect_callback_set(_mosq, &mqtt_client_private_t::lib_connect_callback);
    mosquitto_disconnect_callback_set(_mosq, &mqtt_client_private_t::lib_disconnect_callback);
    mosquitto_reconnect_delay_set(_mosq, 1, 10, true);

    mosquitto_loop_start(_mosq);
}

mqtt_client_private_t::~mqtt_client_private_t()
{
    disconnect();
    mosquitto_loop_stop(_mosq, false);
    mosquitto_destroy(_mosq);
}

int mqtt_client_private_t::connect(const char* host, const int port, const int keepalive)
{
    const auto res = mosquitto_connect(_mosq, host, port, keepalive);
    _connected = (res == MOSQ_ERR_SUCCESS);
    return res;
}

int mqtt_client_private_t::reconnect()
{
    return mosquitto_reconnect(_mosq);
}

int mqtt_client_private_t::disconnect()
{
    return mosquitto_disconnect(_mosq);
}

bool mqtt_client_private_t::is_connected() const noexcept
{
    return _connected;
}

int mqtt_client_private_t::subscribe(const char* sub, const int qos)
{
    return mosquitto_subscribe(_mosq, nullptr, sub, qos);
}

int mqtt_client_private_t::unsubscribe(const char* sub)
{
    return mosquitto_unsubscribe(_mosq, nullptr, sub);
}

int mqtt_client_private_t::publish(
    const char* topic, int* mid, int payloadlen, const void* payload, int qos, bool retain) const
{
    return mosquitto_publish(_mosq, mid, topic, payloadlen, (const void*)payload, qos, retain);
}

int mqtt_client_private_t::receive_callback_set(mqtt_client_t::mqtt_receive_callback_t cbk, void* client)
{
    _rcv_cbk = cbk;
    _rcv_cbk_client = client;
    _rcv_cbk_userp = nullptr;
    return MQTT_ERR_OK;
}

int mqtt_client_private_t::receive_callback_set(
    mqtt_client_t::mqtt_receive_callback_userp_t cbk, void* client, void* userp)
{
    _rcv_cbk = cbk;
    _rcv_cbk_client = client;
    _rcv_cbk_userp = userp;
    return MQTT_ERR_OK;
}

int mqtt_client_private_t::receive_callback_clear()
{
    _rcv_cbk = nullptr;
    _rcv_cbk_client = nullptr;
    _rcv_cbk_userp = nullptr;
    return MQTT_ERR_OK;
}

/** @todo */
template <class... Ts>
struct overload : Ts...
{
    using Ts::operator()...;
};
template <class... Ts>
overload(Ts...) -> overload<Ts...>;

void mqtt_client_private_t::lib_receive_callback(mosquitto*, void* mqtt_client, const mosquitto_message* msg)
{
    decltype(auto) c = static_cast<mqtt_client_private_t*>(mqtt_client);
    mqtt_message_t mqtt_msg{msg->mid, msg->topic, (char*)msg->payload, msg->payloadlen, msg->qos, msg->retain};

    std::visit(
        overload{// do nothing if no callback is set
                 [](std::nullptr_t&) {},
                 // call callback without userdata
                 [&](FLECS::mqtt_client_t::mqtt_receive_callback_t& cbk) {
                     cbk(static_cast<FLECS::mqtt_client_t*>(c->_rcv_cbk_client), &mqtt_msg);
                 },
                 // call callback with userdata
                 [&](FLECS::mqtt_client_t::mqtt_receive_callback_userp_t cbk) {
                     cbk(static_cast<FLECS::mqtt_client_t*>(c->_rcv_cbk_client), &mqtt_msg, c->_rcv_cbk_userp);
                 }},
        c->_rcv_cbk);
}

void mqtt_client_private_t::lib_connect_callback(mosquitto*, void* mqtt_client, int rc)
{
    decltype(auto) c = static_cast<mqtt_client_private_t*>(mqtt_client);
    c->_connected = (rc == 0);
}

int mqtt_client_private_t::disconnect_callback_set(mqtt_client_t::mqtt_disconnect_callback_t cbk, void* client)
{
    _disconnect_cbk = cbk;
    _disconnect_cbk_client = client;
    _disconnect_cbk_userp = nullptr;
    return MQTT_ERR_OK;
}

int mqtt_client_private_t::disconnect_callback_set(
    mqtt_client_t::mqtt_disconnect_callback_userp_t cbk, void* client, void* userp)
{
    _disconnect_cbk = cbk;
    _disconnect_cbk_client = client;
    _disconnect_cbk_userp = userp;
    return MQTT_ERR_OK;
}

int mqtt_client_private_t::disconnect_callback_clear()
{
    _disconnect_cbk = nullptr;
    _disconnect_cbk_client = nullptr;
    _disconnect_cbk_userp = nullptr;
    return MQTT_ERR_OK;
}

void mqtt_client_private_t::lib_disconnect_callback(mosquitto*, void* mqtt_client, int)
{
    decltype(auto) c = static_cast<mqtt_client_private_t*>(mqtt_client);

    c->_connected = false;
    std::visit(
        overload{// do nothing if no callback is set
                 [](std::nullptr_t&) {},
                 // call callback without userdata
                 [&](FLECS::mqtt_client_t::mqtt_disconnect_callback_t& cbk) {
                     cbk(static_cast<FLECS::mqtt_client_t*>(c->_disconnect_cbk_client));
                 },
                 // call callback with userdata
                 [&](FLECS::mqtt_client_t::mqtt_disconnect_callback_userp_t cbk) {
                     cbk(static_cast<FLECS::mqtt_client_t*>(c->_disconnect_cbk_client), c->_disconnect_cbk_userp);
                 }},
        c->_disconnect_cbk);
}

} // namespace Private
} // namespace FLECS
