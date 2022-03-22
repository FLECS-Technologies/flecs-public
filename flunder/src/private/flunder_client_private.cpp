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

#include "private/flunder_client_private.h"

#include <cpr/cpr.h>
#include <json/json.h>

#include <thread>

#include "util/http/status_codes.h"

namespace FLECS {
namespace Private {

flunder_client_private_t::flunder_client_private_t()
    : _json_reader{Json::CharReaderBuilder().newCharReader()}
{}

flunder_client_private_t::~flunder_client_private_t()
{}

int flunder_client_private_t::connect(const std::string_view& /*host*/, int /*port*/)
{
    return 0;
}

int flunder_client_private_t::reconnect()
{
    return 0;
}

int flunder_client_private_t::disconnect()
{
    return 0;
}

int flunder_client_private_t::publish(const std::string_view& path, const std::string& type, const std::string& value)
{
    const auto url = std::string{"http://localhost:8000"}.append(path);
    const auto res = cpr::Put(cpr::Url{url}, cpr::Header{{"content-type", type}}, cpr::Body{value});
    return (res.status_code == 200) ? 0 : -1;
}

// int flunder_client_private_t::subscribe(
//     const std::string_view& /*path*/, const flunder_client_t::subscribe_callback_t& /*cbk*/)
// {
//     return -1;
// }

// int flunder_client_private_t::unsubscribe(const std::string_view& /*path*/)
// {
//     return -1;
// }

auto flunder_client_private_t::get(const std::string_view& path) -> std::tuple<int, std::vector<flunder_variable_t>>
{
    auto vars = std::vector<flunder_variable_t>{};

    const auto url = std::string{"http://localhost:8000"}.append(path);
    const auto res = cpr::Get(cpr::Url{url});

    if (res.status_code != static_cast<long>(http_status_e::Ok))
    {
        return {-1, vars};
    }

    decltype(auto) str = res.text;
    auto json = Json::Value{};
    if (!_json_reader->parse(res.text.c_str(), res.text.c_str() + res.text.length(), &json, nullptr))
    {
        return {-1, vars};
    }

    for (decltype(auto) it = json.begin(); it != json.end(); ++it)
    {
        vars.emplace_back(flunder_variable_t{
            (*it)["key"].as<std::string>(),
            (*it)["value"].as<std::string>(),
            (*it)["encoding"].as<std::string>(),
            (*it)["time"].as<std::string>()});
    }

    return {0, vars};
}

int flunder_client_private_t::erase(const std::string_view& path)
{
    const auto url = std::string{"http://localhost:8000"}.append(path);
    const auto res = cpr::Delete(cpr::Url{url});
    return (res.status_code == 200) ? 0 : -1;
}

} // namespace Private
} // namespace FLECS