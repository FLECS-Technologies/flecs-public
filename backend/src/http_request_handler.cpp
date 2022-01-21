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

#include "backend/http_request_handler.h"

#include <algorithm>
#include <functional>
#include <iostream>
#include <tuple>
#include <utility>

#include "util/container/map_constexpr.h"
#include "util/http/response_headers.h"
#include "util/http/version_strings.h"
#include "util/process/process.h"
#include "util/string/literals.h"

namespace FLECS {

#define REQUIRED_JSON_VALUE(val)                                                              \
    if (_json_value[#val].isNull())                                                           \
    {                                                                                         \
        std::cerr << "Missing field " << #val << " in request" << std::endl;                  \
        return http_status_e::BadRequest;                                                     \
    }                                                                                         \
    auto val = std::string{};                                                                 \
    try                                                                                       \
    {                                                                                         \
        val = _json_value[#val].as<std::string>();                                            \
    } catch (const Json::LogicError& ex)                                                      \
    {                                                                                         \
        std::cerr << "Malformed field " << #val << " in request: " << ex.what() << std::endl; \
        return http_status_e::BadRequest;                                                     \
    }

template <typename... Args>
std::tuple<int, std::string, std::string> run_flecs_service(const char* action, Args&&... args)
{
    process_t proc_install{};
    const auto res = proc_install.spawnp("flecs", "app-manager", action, args...);

    if (res != 0)
    {
        return std::make_tuple(-1, "", "flecs executable not found");
    }

    proc_install.wait(true, true);
    return std::make_tuple(proc_install.exit_code(), proc_install.stdout(), proc_install.stderr());
}

template <typename T>
auto build_response_impl(Json::Value json, const char* field, T&& value)
{
    json[field] = value;
    return json;
}

template <typename T, typename... Args>
auto build_response_impl(Json::Value json, const char* const field, T&& value, Args&&... args)
{
    json[field] = value;
    return build_response_impl(json, args...);
}

template <typename... Args>
auto build_response(Args&&... args)
{
    Json::Value json{};
    return build_response_impl(json, args...);
}

http_request_handler_t::http_request_handler_t(FLECS::tcp_socket_t&& conn_socket)
    : _conn_socket{std::move(conn_socket)}
    , _llhttp_settings{}
    , _llhttp_ext{}
    , _json_builder{}
    , _json_reader{_json_builder.newCharReader()}
    , _json_value{}
    , _json_response{}
{
    llhttp_settings_init(&_llhttp_settings);
    _llhttp_settings.on_body = &llhttp_ext_on_body;
    _llhttp_settings.on_url = &llhttp_ext_on_url;
    _llhttp_settings.on_message_complete = &llhttp_ext_on_message_complete;
    llhttp_init(&_llhttp_ext, HTTP_REQUEST, &_llhttp_settings);
}

http_status_e http_request_handler_t::dispatch()
{
    const auto err = receive_request();
    if (err != http_status_e::Ok)
    {
        return err;
    }

    const auto it = find_backend();
    if (it == _backend_callbacks.end())
    {
        return http_status_e::NotImplemented;
    }

    if (_llhttp_ext.method == HTTP_POST)
    {
        const auto success = _json_reader->parse(
            _llhttp_ext._body.c_str(),
            _llhttp_ext._body.c_str() + _llhttp_ext._body.size(),
            &_json_value,
            nullptr);
        if (!success)
        {
            return http_status_e::BadRequest;
        }
    }

    return std::invoke(it->second, this);
}

int http_request_handler_t::send_response(http_status_e status)
{
    std::stringstream ss;

    // HTTP header
    ss << http_version_1_1 << " " << http_response_header_map.at(status).second;
    // Separator
    ss << "\r\n";
    // Body
    ss << _json_response.toStyledString();

    return _conn_socket.send(ss.str().c_str(), ss.str().length(), 0);
}

http_status_e http_request_handler_t::receive_request()
{
    using FLECS::operator""_kiB;
    char buf[16_kiB];

    const auto size = _conn_socket.recv(buf, sizeof(buf), 0);
    if ((size <= 0) || (llhttp_execute(&_llhttp_ext, buf, size) != HPE_OK))
    {
        return http_status_e::BadRequest;
    }

    return http_status_e::Ok;
}

auto http_request_handler_t::find_backend() -> http_request_handler_t::backend_callback_table_t::const_iterator
{
    const auto pos = _llhttp_ext._url.find_last_of('/');
    if (pos == std::string::npos)
    {
        return _backend_callbacks.cend();
    }
    return _backend_callbacks.find(_llhttp_ext._url.c_str() + pos + 1);
}

http_status_e http_request_handler_t::install_app()
{
    if (_llhttp_ext.method != HTTP_POST)
    {
        return http_status_e::MethodNotAllowed;
    }

    REQUIRED_JSON_VALUE(app);
    REQUIRED_JSON_VALUE(version);

    std::cout << "[Request]: Install " << app << " " << version << std::endl;

    const auto res = run_flecs_service("install", app, version);
    const auto success = (std::get<0>(res) == 0);

    _json_response =
        build_response("app", app, "version", version, "additionalInfo", success ? std::string{} : std::get<2>(res));

    return success ? http_status_e::Ok : http_status_e::InternalServerError;
}

http_status_e http_request_handler_t::uninstall_app()
{
    if (_llhttp_ext.method != HTTP_POST)
    {
        return http_status_e::MethodNotAllowed;
    }

    REQUIRED_JSON_VALUE(app);
    REQUIRED_JSON_VALUE(version);

    std::cout << "[Request]: Uninstall " << app << " " << version << std::endl;

    const auto res = run_flecs_service("uninstall", app, version);
    const auto success = (std::get<0>(res) == 0);

    _json_response =
        build_response("app", app, "version", version, "additionalInfo", success ? std::string{} : std::get<2>(res));

    return success ? http_status_e::Ok : http_status_e::InternalServerError;
}

http_status_e http_request_handler_t::create_app_instance()
{
    if (_llhttp_ext.method != HTTP_POST)
    {
        return http_status_e::MethodNotAllowed;
    }

    REQUIRED_JSON_VALUE(app);
    REQUIRED_JSON_VALUE(version);
    REQUIRED_JSON_VALUE(instanceName);

    std::cout << "[Request]: Create instance " << instanceName << " of " << app << " " << version << std::endl;

    const auto res = run_flecs_service("create-instance", app, version, instanceName);
    const bool success = (std::get<0>(res) == 0);

    _json_response = build_response(
        "app",
        app,
        "version",
        version,
        "instanceId",
        success ? std::get<1>(res) : std::string{},
        "additionalInfo",
        success ? std::string{} : std::get<2>(res));

    return success ? http_status_e::Ok : http_status_e::InternalServerError;
}

http_status_e http_request_handler_t::delete_app_instance()
{
    if (_llhttp_ext.method != HTTP_POST)
    {
        return http_status_e::MethodNotAllowed;
    }

    REQUIRED_JSON_VALUE(app);
    REQUIRED_JSON_VALUE(version);
    REQUIRED_JSON_VALUE(instanceId);

    std::cout << "[Request]: Delete instance " << instanceId << " of " << app << " " << version << std::endl;

    const auto res = run_flecs_service("delete-instance", instanceId, app, version);
    const auto success = (std::get<0>(res) == 0);

    _json_response = build_response(
        "app",
        app,
        "version",
        version,
        "instanceId",
        instanceId,
        "additionalInfo",
        success ? std::string{} : std::get<2>(res));

    return success ? http_status_e::Ok : http_status_e::InternalServerError;
}

http_status_e http_request_handler_t::start_app_instance()
{
    if (_llhttp_ext.method != HTTP_POST)
    {
        return http_status_e::MethodNotAllowed;
    }

    REQUIRED_JSON_VALUE(app);
    REQUIRED_JSON_VALUE(version);
    REQUIRED_JSON_VALUE(instanceId);

    std::cout << "[Request]: Start instance " << instanceId << " of " << app << " " << version << std::endl;

    const auto res = run_flecs_service("start-instance", instanceId, app, version);
    const auto success = (std::get<0>(res) == 0);

    _json_response = build_response(
        "app",
        app,
        "version",
        version,
        "instanceId",
        instanceId,
        "additionalInfo",
        success ? std::string{} : std::get<2>(res));

    return success ? http_status_e::Ok : http_status_e::InternalServerError;
}

http_status_e http_request_handler_t::stop_app_instance()
{
    if (_llhttp_ext.method != HTTP_POST)
    {
        return http_status_e::MethodNotAllowed;
    }

    REQUIRED_JSON_VALUE(app);
    REQUIRED_JSON_VALUE(version);
    REQUIRED_JSON_VALUE(instanceId);

    std::cout << "[Request]: Stop instance " << instanceId << " of " << app << " " << version << std::endl;

    const auto res = run_flecs_service("stop-instance", instanceId, app, version);
    const auto success = (std::get<0>(res) == 0);

    _json_response = build_response(
        "app",
        app,
        "version",
        version,
        "instanceId",
        instanceId,
        "additionalInfo",
        success ? std::string{} : std::get<2>(res));

    return success ? http_status_e::Ok : http_status_e::InternalServerError;
}

http_status_e http_request_handler_t::installed_apps_list()
{
    if (_llhttp_ext.method != HTTP_GET)
    {
        return http_status_e::MethodNotAllowed;
    }

    std::cout << "[Request]: List installed apps" << std::endl;

    const auto res = run_flecs_service("list-apps");
    const auto success = (std::get<0>(res) == 0);

    if (success)
    {
        const auto json_output = std::get<1>(res);

        const auto json_result = _json_reader->parse(
            json_output.c_str(),
            json_output.c_str() + json_output.size(),
            &_json_response,
            nullptr);
        if (json_result)
        {
            _json_response["additionalInfo"] = std::string{};
        }
    }
    return http_status_e::Ok;
}

http_status_e http_request_handler_t::sideload_app()
{
    if (_llhttp_ext.method != HTTP_PUT)
    {
        return http_status_e::MethodNotAllowed;
    }

    std::cout << "[Request]: Sideload app" << std::endl;

    const auto manifest = _llhttp_ext._body;
    char tmp[] = "/tmp/flecs-manifest-XXXXXX";
    const auto fd = mkstemp(tmp);
    if (fd < 0)
    {
        return http_status_e::InternalServerError;
    }
    const auto res = write(fd, _llhttp_ext._body.c_str(), _llhttp_ext._body.size());
    if (res < 0)
    {
        close(fd);
        return http_status_e::InternalServerError;
    }

    const auto flecs_res = run_flecs_service("sideload", tmp);
    const auto success = (std::get<0>(flecs_res) == 0);
    unlink(tmp);
    close(fd);

    if (!success)
    {
        return http_status_e::InternalServerError;
    }

    return http_status_e::Ok;
}

} // namespace FLECS
