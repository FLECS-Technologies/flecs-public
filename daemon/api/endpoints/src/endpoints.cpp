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

#include "endpoints.h"

namespace FLECS {

endpoint_t::endpoint_t()
    : _endpoint{}
    , _method{}
    , _cbk{}
{}

endpoint_t::endpoint_t(const char* endpoint, llhttp_method method, cbk_t cbk)
    : _endpoint{endpoint}
    , _method{method}
    , _cbk{cbk}
{}

endpoint_factory_t& endpoint_factory_t::instance()
{
    static endpoint_factory_t factory;
    return factory;
}

std::optional<endpoint_t> endpoint_factory_t::query(const char* endpoint, llhttp_method method)
{
    decltype(auto) it = std::find_if(_endpoint_table.cbegin(), _endpoint_table.cend(), [&](const endpoint_t& ep) {
        return ((endpoint == ep.endpoint()) && (ep.method() == method));
    });
    if (it != _endpoint_table.end())
    {
        return *it;
    }
    return {};
}

void endpoint_factory_t::register_endpoint(const char* endpoint_regex, llhttp_method method, endpoint_t::cbk_t cbk)
{
    _endpoint_table.emplace_back(endpoint_regex, method, cbk);
}

register_endpoint_t::register_endpoint_t(const char* endpoint_regex, llhttp_method method, endpoint_t::cbk_t cbk)
{
    endpoint_factory_t::instance().register_endpoint(endpoint_regex, method, cbk);
}

namespace api {

void register_endpoint(const char* endpoint_regex, llhttp_method method, endpoint_t::cbk_t cbk)
{
    return endpoint_factory_t::instance().register_endpoint(endpoint_regex, method, cbk);
}

std::optional<endpoint_t> query_endpoint(const char* endpoint, llhttp_method method)
{
    return endpoint_factory_t::instance().query(endpoint, method);
}

} // namespace api

} // namespace FLECS
