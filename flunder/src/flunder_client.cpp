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

#include "flunder_client.h"

#include "private/flunder_client_private.h"

namespace FLECS {

flunder_client_t::flunder_client_t()
    : _impl{new Private::flunder_client_private_t{}}
{}

flunder_client_t::flunder_client_t(flunder_client_t&& other)
    : _impl{std::move(other._impl)}
{}

flunder_client_t& flunder_client_t::operator=(flunder_client_t&& other)
{
    swap(*this, other);
    return *this;
}

flunder_client_t::~flunder_client_t()
{}

auto flunder_client_t::connect() //
    -> int
{
    return connect(FLUNDER_HOST, FLUNDER_PORT);
}

auto flunder_client_t::connect(std::string_view host, int port) //
    -> int
{
    return _impl->connect(host, port);
}

auto flunder_client_t::reconnect() //
    -> int
{
    return _impl->reconnect();
}

auto flunder_client_t::disconnect() //
    -> int
{
    return _impl->disconnect();
}

auto flunder_client_t::publish(std::string_view topic, bool value) //
    -> int
{
    return publish_bool(topic, value ? "true" : "false");
}

auto flunder_client_t::publish(std::string_view topic, const char* value) //
    -> int
{
    return publish_string(topic, std::string{value});
}

auto flunder_client_t::publish(std::string_view topic, const void* data, size_t len) //
    -> int
{
    return publish_raw(topic, data, len);
}

auto flunder_client_t::publish_bool(std::string_view topic, const std::string& value) //
    -> int
{
    return _impl->publish_bool(topic, value);
}

auto flunder_client_t::publish_int(std::string_view topic, size_t size, bool is_signed, const std::string& value) //
    -> int
{
    return _impl->publish_int(topic, size, is_signed, value);
}

auto flunder_client_t::publish_float(std::string_view topic, size_t size, const std::string& value) //
    -> int
{
    return _impl->publish_float(topic, size, value);
}

auto flunder_client_t::publish_string(std::string_view topic, const std::string& value) //
    -> int
{
    return _impl->publish_string(topic, value);
}

auto flunder_client_t::publish_raw(std::string_view topic, const void* data, size_t len) //
    -> int
{
    return _impl->publish_raw(topic, data, len);
}

auto flunder_client_t::subscribe(std::string_view topic, subscribe_cbk_t cbk) //
    -> int
{
    return _impl->subscribe(this, topic, cbk);
}

auto flunder_client_t::subscribe(std::string_view topic, subscribe_cbk_userp_t cbk, const void* userp) //
    -> int
{
    return _impl->subscribe(this, topic, cbk, userp);
}

auto flunder_client_t::unsubscribe(std::string_view topic) //
    -> int
{
    return _impl->unsubscribe(topic);
}

auto flunder_client_t::add_mem_storage(std::string_view name, std::string_view topic) //
    -> int
{
    return _impl->add_mem_storage(std::string{name}, topic);
}

auto flunder_client_t::remove_mem_storage(std::string_view name) //
    -> int
{
    return _impl->remove_mem_storage(std::string{name});
}

auto flunder_client_t::get(std::string_view topic) -> std::tuple<int, std::vector<flunder_variable_t>>
{
    return _impl->get(topic);
}

auto flunder_client_t::erase(std::string_view topic) //
    -> int
{
    return _impl->erase(topic);
}

auto swap(flunder_client_t& lhs, flunder_client_t& rhs) noexcept //
    -> void
{
    using std::swap;
    swap(lhs._impl, rhs._impl);
}

extern "C" {

FLECS_EXPORT void* flunder_client_new(void)
{
    return static_cast<void*>(new FLECS::flunder_client_t{});
}

FLECS_EXPORT void flunder_client_destroy(void* flunder)
{
    delete static_cast<FLECS::flunder_client_t*>(flunder);
}

FLECS_EXPORT int flunder_connect(void* flunder, const char* host, int port)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->connect(host, port);
}

FLECS_EXPORT int flunder_reconnect(void* flunder)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->reconnect();
}

FLECS_EXPORT int flunder_disconnect(void* flunder)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->disconnect();
}

FLECS_EXPORT int flunder_subscribe(void* flunder, const char* topic, flunder_subscribe_cbk_t cbk)
{
    auto p = reinterpret_cast<void (*)(flunder_client_t*, const flunder_variable_t*)>(cbk);
    return static_cast<FLECS::flunder_client_t*>(flunder)->subscribe(topic, p);
}
FLECS_EXPORT int flunder_subscribe_userp(
    void* flunder, const char* topic, flunder_subscribe_cbk_userp_t cbk, const void* userp)
{
    auto p = reinterpret_cast<void (*)(flunder_client_t*, const flunder_variable_t*, const void*)>(cbk);
    return static_cast<FLECS::flunder_client_t*>(flunder)->subscribe(topic, p, userp);
}

FLECS_EXPORT int flunder_unsubscribe(void* flunder, const char* topic)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->unsubscribe(topic);
}

FLECS_EXPORT int flunder_publish_bool(void* flunder, const char* topic, bool value)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->publish(topic, value);
}

FLECS_EXPORT int flunder_publish_int(void* flunder, const char* topic, int value)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->publish(topic, value);
}

FLECS_EXPORT int flunder_publish_float(void* flunder, const char* topic, float value)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->publish(topic, value);
}

FLECS_EXPORT int flunder_publish_double(void* flunder, const char* topic, double value)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->publish(topic, value);
}

FLECS_EXPORT int flunder_publish_string(void* flunder, const char* topic, const char* value)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->publish(topic, value);
}

FLECS_EXPORT int flunder_publish_raw(void* flunder, const char* topic, const void* value, size_t payloadlen)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->publish(topic, value, payloadlen);
}

FLECS_EXPORT int flunder_add_mem_storage(void* flunder, const char* name, const char* topic)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->add_mem_storage(name, topic);
}

FLECS_EXPORT int flunder_remove_mem_storage(void* flunder, const char* name)
{
    return static_cast<FLECS::flunder_client_t*>(flunder)->remove_mem_storage(name);
}

} // extern "C"

} // namespace FLECS
