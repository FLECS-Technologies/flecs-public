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

#include <json/json.h>

#include "endpoints/endpoints.h"
#include "util/http/status_codes.h"

namespace FLECS {

// Helper macros to parse JSON arguments passed to endpoints
#define REQUIRED_JSON_VALUE(json, val)                                                       \
    if (json[#val].isNull())                                                                 \
    {                                                                                        \
        response["additionalInfo"] = std::string{"Missing field "} + #val + " in request";   \
        return http_status_e::BadRequest;                                                    \
    }                                                                                        \
    auto val = std::string{};                                                                \
    try                                                                                      \
    {                                                                                        \
        val = json[#val].as<std::string>();                                                  \
    }                                                                                        \
    catch (const Json::LogicError& ex)                                                       \
    {                                                                                        \
        response["additionalInfo"] = std::string{"Malformed field "} + #val + " in request"; \
        return http_status_e::BadRequest;                                                    \
    }

#define OPTIONAL_JSON_VALUE(json, val)          \
    auto val = std::string{};                   \
    if (!json[#val].isNull())                   \
    {                                           \
        try                                     \
        {                                       \
            val = json[#val].as<std::string>(); \
        }                                       \
        catch (const Json::LogicError& ex)      \
        {                                       \
        }                                       \
    }

// Module base class - tbd
class module_t
{
public:
    virtual ~module_t() = default;

    void init();
    // std::string usage();

private:
    virtual void do_init(){};
    // virtual std::string do_usage() = 0;
};

} // namespace FLECS

#endif // DB1BEA7C_952B_4D0F_A1A5_3DD71D6CB69B
