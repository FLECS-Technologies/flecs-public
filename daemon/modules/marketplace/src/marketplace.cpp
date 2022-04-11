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

#include "marketplace.h"

#include "factory/factory.h"

namespace FLECS {

namespace {
register_module_t<module_marketplace_t> _reg("mp");
}

module_marketplace_t::module_marketplace_t()
{
    using namespace std::placeholders;

    api::register_endpoint("/marketplace/login", std::bind(&module_marketplace_t::mp_login, this, _1, _2));
    api::register_endpoint("/marketplace/logout", std::bind(&module_marketplace_t::mp_logout, this, _1, _2));
}

http_status_e module_marketplace_t::mp_login(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, user);
    REQUIRED_JSON_VALUE(args, token);

    _user = user;
    _token = token;

    response["additionalInfo"] = "OK";

    return http_status_e::Ok;
}

http_status_e module_marketplace_t::mp_logout(const Json::Value& args, Json::Value& response)
{
    OPTIONAL_JSON_VALUE(args, user);

    _user.clear();
    _token.clear();

    response["additionalInfo"] = "OK";

    return http_status_e::Ok;
}

} // namespace FLECS
