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

#ifndef FLECS_ui_backend_http_request_handler_h
#define FLECS_ui_backend_http_request_handler_h

#include <cstring>
#include <string_view>
#include <utility>

#include "external/jsoncpp-1.9.5/include/json/json.h"
#include "util/container/map_constexpr.h"
#include "util/http/status_codes.h"
#include "util/llhttp_ext/llhttp_ext.h"
#include "util/socket/socket.h"

namespace FLECS {

class http_request_handler_t
{
public:
    explicit http_request_handler_t(FLECS::tcp_socket_t&& conn_socket);

    http_status_e dispatch();
    int send_response(http_status_e status);

private:
    using backend_callback_t = http_status_e (http_request_handler_t::*)();
    using backend_callback_table_t = map_c<const char*, backend_callback_t, 7, string_comparator>;

    http_status_e receive_request();
    auto find_backend() -> backend_callback_table_t::const_iterator;

    http_status_e install_app();
    http_status_e uninstall_app();
    http_status_e create_app_instance();
    http_status_e delete_app_instance();
    http_status_e start_app_instance();
    http_status_e stop_app_instance();
    http_status_e installed_apps_list();

    static constexpr backend_callback_table_t _backend_callbacks = {{
        std::make_pair("InstallApp", &http_request_handler_t::install_app),
        std::make_pair("UninstallApp", &http_request_handler_t::uninstall_app),
        std::make_pair("CreateAppInstance", &http_request_handler_t::create_app_instance),
        std::make_pair("DeleteAppInstance", &http_request_handler_t::delete_app_instance),
        std::make_pair("StartAppInstance", &http_request_handler_t::start_app_instance),
        std::make_pair("StopAppInstance", &http_request_handler_t::stop_app_instance),
        std::make_pair("InstalledAppList", &http_request_handler_t::installed_apps_list),
    }};

    FLECS::tcp_socket_t _conn_socket;
    llhttp_settings_t _llhttp_settings;
    FLECS::llhttp_ext_t _llhttp_ext;

    Json::CharReaderBuilder _json_builder;
    std::unique_ptr<Json::CharReader> _json_reader;
    Json::Value _json_value;
    Json::Value _json_response;
};

} // namespace FLECS

#endif // FLECS_ui_backend_http_request_handler_h
