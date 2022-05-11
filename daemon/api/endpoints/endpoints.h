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

#ifndef D2CA0BA7_23AC_45FE_B6BF_DC1EFD9E8446
#define D2CA0BA7_23AC_45FE_B6BF_DC1EFD9E8446

#include <llhttp.h>

#include <functional>
#include <optional>
#include <string>
#include <vector>

#include "util/http/status_codes.h"
#include "util/string/comparator.h"

namespace Json {
class Value;
} // namespace Json

namespace FLECS {

class endpoint_t
{
public:
    using cbk_t = std::function<http_status_e(const Json::Value&, Json::Value&)>;

    endpoint_t();
    endpoint_t(const char*, llhttp_method method, cbk_t cbk);

    auto operator()(const Json::Value& args, Json::Value& response) const { return _cbk(args, response); }

    auto& endpoint() const noexcept { return _endpoint; }
    auto& method() const noexcept { return _method; }

private:
    std::string _endpoint;
    llhttp_method _method;
    cbk_t _cbk;
};

class endpoint_factory_t
{
public:
    static endpoint_factory_t& instance();

    void register_endpoint(const char* endpoint_regex, llhttp_method method, endpoint_t::cbk_t cbk);
    std::optional<endpoint_t> query(const char* endpoint_regex, llhttp_method method);

private:
    friend struct register_endpoint_t;

    endpoint_factory_t() = default;

    endpoint_factory_t(const endpoint_factory_t&) = delete;
    endpoint_factory_t(endpoint_factory_t&&) = delete;
    endpoint_factory_t& operator=(endpoint_factory_t) = delete;

    using endpoint_table_t = std::vector<endpoint_t>;
    endpoint_table_t _endpoint_table;
};

struct register_endpoint_t
{
    register_endpoint_t(const char* endpoint_regex, llhttp_method method, endpoint_t::cbk_t cbk);
};

namespace api {
void register_endpoint(const char* endpoint_regex, llhttp_method method, endpoint_t::cbk_t cbk);
std::optional<endpoint_t> query_endpoint(const char* endpoint, llhttp_method method);
} // namespace api

} // namespace FLECS

#endif // D2CA0BA7_23AC_45FE_B6BF_DC1EFD9E8446
