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

#include "api.h"

#include <poll.h>

#include <cstdio>
#include <nlohmann/json.hpp>
#include <thread>
#include <vector>

#include "endpoints/endpoints.h"
#include "util/http/response_headers.h"
#include "util/http/version_strings.h"
#include "util/llhttp_ext/llhttp_ext.h"
#include "util/signal_handler/signal_handler.h"
#include "util/string/literals.h"

namespace FLECS {

flecs_api_t::flecs_api_t()
    : _tcp_server{8951, INADDR_ANY, 10}
    , _unix_server{"/var/run/flecs/flecs.sock", 10}
{
    if (!_tcp_server.is_running() || !_unix_server.is_running())
    {
        std::terminate();
    }
}

flecs_api_t::~flecs_api_t()
{}

int flecs_api_t::run()
{
    do
    {
        pollfd pollfds[2] = {{_unix_server.fd(), POLLIN, 0}, {_tcp_server.fd(), POLLIN, 0}};

        auto res = poll(pollfds, sizeof(pollfds) / (sizeof(pollfds[0])), -1);

        if (res > 0)
        {
            if (pollfds[0].revents == POLLIN)
            {
                auto conn_socket = unix_socket_t{_unix_server.accept(nullptr, nullptr)};
                if (conn_socket.is_valid())
                {
                    process(conn_socket);
                }
            }
            if (pollfds[1].revents == POLLIN)
            {
                auto conn_socket = tcp_socket_t{_tcp_server.accept(nullptr, nullptr)};
                if (conn_socket.is_valid())
                {
                    process(conn_socket);
                }
            }
        }
    } while (!g_stop);

    return 0;
}

http_status_e flecs_api_t::process(socket_t& conn_socket)
{
    // Receive data from the connected client
    using FLECS::operator""_kiB;
    char buf[128_kiB];

    auto n_bytes = conn_socket.recv(buf, sizeof(buf), 0);
    if (n_bytes <= 0)
    {
        return http_status_e::BadRequest;
    }

    auto llhttp_ext = llhttp_ext_t{};
    auto llhttp_ext_settings = llhttp_settings_t{};
    llhttp_ext_settings_init(&llhttp_ext_settings);
    llhttp_init(&llhttp_ext, HTTP_REQUEST, &llhttp_ext_settings);
    if (llhttp_execute(&llhttp_ext, buf, n_bytes) != HPE_OK)
    {
        return http_status_e::BadRequest;
    }

    auto args = nlohmann::json{};
    if (llhttp_ext.method == HTTP_POST || llhttp_ext.method == HTTP_PUT)
    {
        args = nlohmann::json::parse(llhttp_ext._body, nullptr, false);
        if (args.is_discarded())
        {
            return http_status_e::BadRequest;
        }
    }
    else if (llhttp_ext.method != HTTP_GET)
    {
        return http_status_e::MethodNotAllowed;
    }

    http_status_e err = http_status_e::NotImplemented;

    const auto endpoint = api::query_endpoint(llhttp_ext._url.c_str(), static_cast<llhttp_method>(llhttp_ext.method));
    auto json_response = nlohmann::json{};
    if (endpoint.has_value())
    {
        err = endpoint.value()(args, json_response);
    }

    decltype(auto) body = json_response.dump();
    std::stringstream ss;
    // HTTP header
    ss << http_version_1_1 << " " << http_response_header_map.at(err).second;
    // Content-Type
    ss << "Content-Type: application/json\r\n";
    // Content-Length
    ss << "Content-Length: " << body.length() << "\r\n";
    // Separator
    ss << "\r\n";
    // Body
    ss << body;

    conn_socket.send(ss.str().c_str(), ss.str().length(), 0);

    return err;
}

} // namespace FLECS
