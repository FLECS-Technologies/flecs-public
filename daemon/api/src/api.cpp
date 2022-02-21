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

#include <cstdio>
#include <thread>
#include <vector>

#include "endpoints/endpoints.h"
#include "json/json.h"
#include "signal_handler.h"
#include "util/http/response_headers.h"
#include "util/http/version_strings.h"
#include "util/llhttp_ext/llhttp_ext.h"
#include "util/string/literals.h"

namespace FLECS {

flecs_api_t::flecs_api_t()
    : _server{8951, INADDR_ANY, 1}
    , _json_reader{Json::CharReaderBuilder().newCharReader()}
{
    if (!_server.is_running())
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
        auto conn_socket = tcp_socket_t{_server.accept(nullptr, nullptr)};
        if (conn_socket.is_valid())
        {
            process(conn_socket);
        }
    } while (!g_stop);

    return 0;
}

http_status_e flecs_api_t::process(tcp_socket_t& conn_socket)
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

    auto args = Json::Value{};
    if (llhttp_ext.method == HTTP_POST)
    {
        const auto success = _json_reader->parse(
            llhttp_ext._body.c_str(),
            llhttp_ext._body.c_str() + llhttp_ext._body.size(),
            &args,
            nullptr);
        if (!success)
        {
            return http_status_e::BadRequest;
        }
    }
    else if (llhttp_ext.method == HTTP_PUT)
    {
        char tmp[] = "/tmp/flecs-XXXXXX";
        int fd = mkstemp(tmp);
        if (fd < -1)
        {
            return http_status_e::InternalServerError;
        }
        const auto res = write(fd, llhttp_ext._body.c_str(), llhttp_ext._body.length());
        close(fd);
        if (res != llhttp_ext._body.length())
        {
            return http_status_e::InternalServerError;
        }
        args["path"] = tmp;
    }
    else if (llhttp_ext.method != HTTP_GET)
    {
        return http_status_e::MethodNotAllowed;
    }

    http_status_e err = http_status_e::NotImplemented;

    const auto endpoint = api::query_endpoint(llhttp_ext._url.c_str());
    auto json_response = Json::Value{};
    if (endpoint.has_value())
    {
        err = static_cast<http_status_e>(std::invoke(endpoint.value(), args, json_response));
    }

    std::stringstream ss;
    // HTTP header
    ss << http_version_1_1 << " " << http_response_header_map.at(err).second;
    // Separator
    ss << "\r\n";
    // Body
    ss << json_response.toStyledString();

    conn_socket.send(ss.str().c_str(), ss.str().length(), 0);

    return err;
}

} // namespace FLECS