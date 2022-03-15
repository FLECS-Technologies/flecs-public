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

flunder_client_t::~flunder_client_t()
{}

int flunder_client_t::connect()
{
    return connect(FLUNDER_HOST, FLUNDER_PORT);
}

int flunder_client_t::connect(const std::string_view& host, int port)
{
    return _impl->connect(host, port);
}

int flunder_client_t::reconnect()
{
    return _impl->reconnect();
}

int flunder_client_t::disconnect()
{
    return _impl->disconnect();
}

int flunder_client_t::publish(const std::string_view& path, const char* value)
{
    return publish_string(path, std::string{value});
}

int flunder_client_t::publish(const std::string_view& path, const void* data, size_t len)
{
    return _impl->publish(path, "application/octet-stream", std::string{(const char*)data, len});
}

int flunder_client_t::publish_int(const std::string_view& path, const std::string& value)
{
    return _impl->publish(path, "application/integer", value);
}

int flunder_client_t::publish_float(const std::string_view& path, const std::string& value)
{
    return _impl->publish(path, "application/float", value);
}

int flunder_client_t::publish_string(const std::string_view& path, const std::string& value)
{
    return _impl->publish(path, "text/plain", value);
}

// int flunder_client_t::subscribe(const std::string_view& path, const subscribe_callback_t& cbk)
// {
//     return _impl->subscribe(path, cbk);
// }

// int flunder_client_t::unsubscribe(const std::string_view& path)
// {
//     return _impl->unsubscribe(path);
// }

auto flunder_client_t::get(const std::string_view& path) -> std::tuple<int, std::vector<flunder_variable_t>>
{
    return _impl->get(path);
}

int flunder_client_t::erase(const std::string_view& path)
{
    return _impl->erase(path);
}

} // namespace FLECS
