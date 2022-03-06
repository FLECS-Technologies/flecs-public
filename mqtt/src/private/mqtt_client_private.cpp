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

#include "mqtt_errors.h"

namespace FLECS {
namespace Private {

mqtt_client_private_t::mqtt_client_private_t()
    : _mosq{}
{
    {
        std::lock_guard<std::mutex> lock(_ref_mutex);
        if (!_ref_count)
        {
            mosquitto_lib_init();
        }
        ++_ref_count;
    }

    char hostname[HOST_NAME_MAX + 1]{};
    gethostname(hostname, HOST_NAME_MAX);
    _mosq = mosquitto_new((std::strlen(hostname) == 0) ? nullptr : hostname, true, this);

    mosquitto_message_callback_set(_mosq, &mqtt_client_private_t::lib_receive_callback);

    mosquitto_loop_start(_mosq);
}

mqtt_client_private_t::~mqtt_client_private_t()
{
    disconnect();
    mosquitto_loop_stop(_mosq, false);
    mosquitto_destroy(_mosq);

    std::lock_guard<std::mutex> lock(_ref_mutex);
    --_ref_count;
    if (!_ref_count)
    {
        mosquitto_lib_cleanup();
    }
}

int mqtt_client_private_t::connect(const char* host, const int port, const int keepalive)
{
    return mosquitto_connect(_mosq, host, port, keepalive);
}

int mqtt_client_private_t::reconnect()
{
    return mosquitto_reconnect(_mosq);
}

int mqtt_client_private_t::disconnect()
{
    return mosquitto_disconnect(_mosq);
}

int mqtt_client_private_t::subscribe(const char* sub, const int qos)
{
    return mosquitto_subscribe(_mosq, nullptr, sub, qos);
}

int mqtt_client_private_t::unsubscribe(const char* sub)
{
    return mosquitto_unsubscribe(_mosq, nullptr, sub);
}

int mqtt_client_private_t::publish(const char* topic, int payloadlen, const char* payload, int qos, bool retain)
{
    return mosquitto_publish(_mosq, nullptr, topic, payloadlen, (const void*)payload, qos, retain);
}

int mqtt_client_private_t::receive_callback_set(mqtt_client_t::mqtt_callback_t cbk, void* client)
{
    _rcv_cbk = cbk;
    _rcv_cbk_client = client;
    _rcv_cbk_userp = nullptr;
    return MQTT_ERR_OK;
}

int mqtt_client_private_t::receive_callback_set(mqtt_client_t::mqtt_callback_userp_t cbk, void* client, void* userp)
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
                 [&](FLECS::mqtt_client_t::mqtt_callback_t& cbk) {
                     cbk(static_cast<FLECS::mqtt_client_t*>(c->_rcv_cbk_client), &mqtt_msg);
                 },
                 // call callback with userdata
                 [&](FLECS::mqtt_client_t::mqtt_callback_userp_t cbk) {
                     cbk(static_cast<FLECS::mqtt_client_t*>(c->_rcv_cbk_client), &mqtt_msg, c->_rcv_cbk_userp);
                 }},
        c->_rcv_cbk);
}

} // namespace Private
} // namespace FLECS
