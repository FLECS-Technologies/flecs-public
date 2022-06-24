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

#ifndef DB1BEA7C_952B_4D0F_A1A5_3DD71D6CB69B
#define DB1BEA7C_952B_4D0F_A1A5_3DD71D6CB69B

#include "api/api.h"
#include "util/http/status_codes.h"
#include "util/json/json.h"

namespace FLECS {

// Helper macros to parse JSON arguments passed to endpoints
#define REQUIRED_TYPED_JSON_VALUE(json, val, type)                                           \
    if (!json.contains(#val))                                                                \
    {                                                                                        \
        response["additionalInfo"] = std::string{"Missing field "} + #val + " in request";   \
        return crow::response{crow::status::BAD_REQUEST, response.dump()};                   \
    }                                                                                        \
    auto val = type{};                                                                       \
    try                                                                                      \
    {                                                                                        \
        val = json[#val].get<type>();                                                        \
    }                                                                                        \
    catch (const nlohmann::detail::exception& ex)                                            \
    {                                                                                        \
        response["additionalInfo"] = std::string{"Malformed field "} + #val + " in request"; \
        return crow::response{crow::status::BAD_REQUEST, response.dump()};                   \
    }

#define REQUIRED_JSON_VALUE(json, val) REQUIRED_TYPED_JSON_VALUE(json, val, std::string)

#define OPTIONAL_TYPED_JSON_VALUE(json, val, type)    \
    auto val = type{};                                \
    if (json.contains(#val))                          \
    {                                                 \
        try                                           \
        {                                             \
            val = json[#val].get<std::string>();      \
        }                                             \
        catch (const nlohmann::detail::exception& ex) \
        {                                             \
        }                                             \
    }

#define OPTIONAL_JSON_VALUE(json, val) OPTIONAL_TYPED_JSON_VALUE(json, val, std::string)

// Module base class - tbd
class module_t
{
public:
    virtual ~module_t() = default;

    auto init() -> //
        void;
    // std::string usage();

private:
    virtual auto do_init() //
        -> void = 0;
    // virtual std::string do_usage() = 0;
};

} // namespace FLECS

#endif // DB1BEA7C_952B_4D0F_A1A5_3DD71D6CB69B
