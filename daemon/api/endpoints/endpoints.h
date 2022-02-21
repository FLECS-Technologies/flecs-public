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

#include <functional>
#include <map>
#include <optional>

#include "util/http/status_codes.h"
#include "util/string/comparator.h"

#ifndef FLECS_daemon_api_endpoint_factory_h
#define FLECS_daemon_api_endpoint_factory_h

namespace Json {
class Value;
} // namespace Json

namespace FLECS {

using endpoint_t = std::function<http_status_e(const Json::Value&, Json::Value&)>;

class endpoint_factory_t
{
public:
    static endpoint_factory_t& instance();

    void register_endpoint(const char* endpoint, endpoint_t);
    std::optional<endpoint_t> query(const char* endpoint);

private:
    friend struct register_endpoint_t;

    endpoint_factory_t() = default;

    endpoint_factory_t(const endpoint_factory_t&) = delete;
    endpoint_factory_t(endpoint_factory_t&&) = delete;
    endpoint_factory_t& operator=(endpoint_factory_t) = delete;

    using endpoint_table_t = std::map<const char*, endpoint_t, string_comparator_t>;
    endpoint_table_t _endpoint_table;
};

struct register_endpoint_t
{
    register_endpoint_t(const char* endpoint, endpoint_t cbk);
};

namespace api {
void register_endpoint(const char* endpoint, endpoint_t cbk);
std::optional<endpoint_t> query_endpoint(const char* endpoint);
} // namespace api

} // namespace FLECS

#endif // FLECS_daemon_api_endpoint_factory_h
