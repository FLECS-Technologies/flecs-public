// Copyright 2021 FLECS Technologies GmbH
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

#include "ui/backend/http_request_handler.h"

#include "util/http/response_headers.h"
#include "util/http/version_strings.h"
#include "util/literals.h"
#include "util/map_constexpr.h"
#include "util/process/process.h"

#include <algorithm>
#include <array>
#include <functional>
#include <iostream>
#include <memory>
#include <utility>

namespace FLECS
{

http_request_handler_t::http_request_handler_t(FLECS::tcp_socket_t&& conn_socket)
    : _conn_socket { std::move(conn_socket) }
    , _llhttp_settings {}
    , _llhttp_ext {}
    , _json_builder {}
    , _json_reader { _json_builder.newCharReader() }
    , _json_value {}
    , _additionalInfo {}
{
    llhttp_settings_init(&_llhttp_settings);
    _llhttp_settings.on_body = &llhttp_ext_on_body;
    _llhttp_settings.on_url = &llhttp_ext_on_url;
    _llhttp_settings.on_message_complete = &llhttp_ext_on_message_complete;
    llhttp_init(&_llhttp_ext, HTTP_REQUEST, &_llhttp_settings);
}

http_status_e http_request_handler_t::dispatch()
{
    auto err = receive_request();
    if (err != http_status_e::Ok)
    {
        return err;
    }

    if ((_llhttp_ext.method != HTTP_GET) && (_llhttp_ext.method != HTTP_POST))
    {
        return http_status_e::MethodNotAllowed;
    }

    auto it = find_backend();
    if (it == _backend_callbacks.end())
    {
        return http_status_e::NotImplemented;
    }

    const auto success = _json_reader->parse(
        _llhttp_ext._body.c_str(),
        _llhttp_ext._body.c_str() + _llhttp_ext._body.size(),
        &_json_value,
        nullptr);

    if (!success)
    {
        return http_status_e::BadRequest;
    }

    return std::invoke(it->second, this);
}

int http_request_handler_t::send_response(http_status_e status)
{
    std::stringstream ss;
    ss << http_version_1_1 << " " << http_response_header_map.at(status).second;
    ss << "\r\n";
    ss << "{";
    ss << "\"status\":\"" << (status == http_status_e::Ok ? "success" : "failed") << "\",";
    ss << "\"additionalInfo\":\"" << _additionalInfo << "\"";
    ss << "}\r\n";
    return _conn_socket.send(ss.str().c_str(), ss.str().length(), 0);
}

http_status_e http_request_handler_t::receive_request()
{
    using FLECS::operator""_kiB;
    char buf[16_kiB];

    ssize_t size = _conn_socket.recv(buf, sizeof(buf), 0);
    if ((size <= 0) || (llhttp_execute(&_llhttp_ext, buf, size) != HPE_OK))
    {
        return http_status_e::BadRequest;
    }

    return http_status_e::Ok;
}

auto http_request_handler_t::find_backend() -> http_request_handler_t::backend_callback_table_t::const_iterator
{
    auto pos = _llhttp_ext._url.find_last_of('/');
    if (pos == std::string::npos)
    {
        return _backend_callbacks.cend();
    }
    return _backend_callbacks.find(_llhttp_ext._url.c_str() + pos + 1);
}

http_status_e http_request_handler_t::install_app()
{
    if (_json_value["appId"].isNull())
    {
        _additionalInfo.append("Missing field appId in request");
        return http_status_e::BadRequest;
    }

    if (_json_value["appVersion"].isNull())
    {
        _additionalInfo.append("Missing field appVersion in request");
        return http_status_e::BadRequest;
    }

    std::string appId = _json_value["appId"].as<std::string>();
    std::string version = _json_value["appVersion"].as<std::string>();
    std::cout << "[Request]: Install " << appId << " " << version << std::endl;

    process_t proc_install {};
    auto res = proc_install.spawnp("flecs", "app-manager", "install", appId, version);

    if (res != 0)
    {
        _additionalInfo.append("flecs executable not found");
        return http_status_e::InternalServerError;
    }

    proc_install.wait(true, true);
    if (proc_install.exit_code() != 0)
    {
        _additionalInfo.append(proc_install.stderr());
        return http_status_e::InternalServerError;
    }

    _additionalInfo.append(proc_install.stdout());
    return http_status_e::Ok;
}

http_status_e http_request_handler_t::create_app_instance()
{
    return http_status_e::Ok;
}

http_status_e http_request_handler_t::start_app_instance()
{
    return http_status_e::Ok;
}

http_status_e http_request_handler_t::installed_apps_list()
{
    return http_status_e::Ok;
}

} // namespace FLECS
