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

#include "mqtt_client.h"

#include <algorithm>
#include <cstring>
#include <string>

#include "private/mqtt_client_private.h"

namespace FLECS {

mqtt_client_t::mqtt_client_t()
    : _impl{new Private::mqtt_client_private_t{}}
{}

mqtt_client_t::mqtt_client_t(mqtt_client_t&& other)
    : _impl{std::move(other._impl)}
{}

mqtt_client_t& mqtt_client_t::operator=(mqtt_client_t&& other)
{
    swap(*this, other);
    return *this;
}

mqtt_client_t::~mqtt_client_t()
{
    disconnect();
}

int mqtt_client_t::connect()
{
    return connect(MQTT_HOST, MQTT_PORT, MQTT_KEEPALIVE);
}

int mqtt_client_t::connect(const char* host, int port, int keepalive)
{
    return _impl->connect(host, port, keepalive);
}

int mqtt_client_t::reconnect()
{
    return _impl->reconnect();
}

int mqtt_client_t::disconnect()
{
    return _impl->disconnect();
}

bool mqtt_client_t::is_connected()
{
    return _impl->is_connected();
}

int mqtt_client_t::subscribe(const char* sub, int qos)
{
    return _impl->subscribe(sub, qos);
}

int mqtt_client_t::unsubscribe(const char* sub)
{
    return _impl->unsubscribe(sub);
}

int mqtt_client_t::publish(const char* topic, int payloadlen, const void* payload, int qos, bool retain)
{
    return _impl->publish(topic, payloadlen, payload, qos, retain);
}

int mqtt_client_t::receive_callback_set(mqtt_receive_callback_t cbk)
{
    return _impl->receive_callback_set(cbk, static_cast<void*>(this));
}

int mqtt_client_t::receive_callback_set(mqtt_receive_callback_userp_t cbk, void* userp)
{
    return _impl->receive_callback_set(cbk, static_cast<void*>(this), userp);
}

int mqtt_client_t::receive_callback_clear()
{
    return _impl->receive_callback_clear();
}

int mqtt_client_t::disconnect_callback_set(mqtt_disconnect_callback_t cbk)
{
    return _impl->disconnect_callback_set(cbk, static_cast<void*>(this));
}

int mqtt_client_t::disconnect_callback_set(mqtt_disconnect_callback_userp_t cbk, void* userp)
{
    return _impl->disconnect_callback_set(cbk, static_cast<void*>(this), userp);
}

int mqtt_client_t::disconnect_callback_clear()
{
    return _impl->disconnect_callback_clear();
}

void swap(mqtt_client_t& lhs, mqtt_client_t& rhs) noexcept
{
    using std::swap;
    swap(lhs._impl, rhs._impl);
}

extern "C" {

FLECS_EXPORT void* flecs_mqtt_client_new(void)
{
    return static_cast<void*>(new FLECS::mqtt_client_t{});
}

FLECS_EXPORT void flecs_mqtt_client_destroy(void* mqtt)
{
    delete static_cast<FLECS::mqtt_client_t*>(mqtt);
}

FLECS_EXPORT int flecs_mqtt_connect(void* mqtt, const char* host, int port, int keepalive)
{
    return static_cast<FLECS::mqtt_client_t*>(mqtt)->connect(host, port, keepalive);
}

FLECS_EXPORT int flecs_mqtt_reconnect(void* mqtt)
{
    return static_cast<FLECS::mqtt_client_t*>(mqtt)->reconnect();
}

FLECS_EXPORT int flecs_mqtt_disconnect(void* mqtt)
{
    return static_cast<FLECS::mqtt_client_t*>(mqtt)->disconnect();
}

FLECS_EXPORT bool flecs_mqtt_is_connected(void* mqtt)
{
    return static_cast<FLECS::mqtt_client_t*>(mqtt)->is_connected();
}

FLECS_EXPORT int flecs_mqtt_subscribe(void* mqtt, const char* sub, int qos)
{
    return static_cast<FLECS::mqtt_client_t*>(mqtt)->subscribe(sub, qos);
}

FLECS_EXPORT int flecs_mqtt_unsubscribe(void* mqtt, const char* sub)
{
    return static_cast<FLECS::mqtt_client_t*>(mqtt)->unsubscribe(sub);
}

FLECS_EXPORT int flecs_mqtt_publish(
    void* mqtt, const char* topic, int payloadlen, const void* payload, int qos, bool retain)
{
    return static_cast<FLECS::mqtt_client_t*>(mqtt)->publish(topic, payloadlen, payload, qos, retain);
}

FLECS_EXPORT int flecs_mqtt_receive_callback_set(void* mqtt, flecs_mqtt_callback cbk, void* userp)
{
    auto p = reinterpret_cast<void (*)(FLECS::mqtt_client_t*, FLECS::mqtt_message_t*, void*)>(cbk);
    return static_cast<FLECS::mqtt_client_t*>(mqtt)->receive_callback_set(p, userp);
}

FLECS_EXPORT int flecs_mqtt_receive_callback_clear(void* mqtt)
{
    return static_cast<FLECS::mqtt_client_t*>(mqtt)->receive_callback_clear();
}

} // extern "C"

} // namespace FLECS
