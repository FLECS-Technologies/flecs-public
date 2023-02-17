// Copyright 2021-2023 FLECS Technologies GmbH
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

#pragma once

#include "api/api.h"
#include "core/flecs.h"
#include "util/fs/fs.h"
#include "util/json/json.h"

namespace FLECS {

// Helper macros to parse JSON arguments passed to endpoints
#define REQUIRED_TYPED_JSON_VALUE(json, val, type)                                           \
    if (!json.contains(#val)) {                                                              \
        response["additionalInfo"] = std::string{"Missing field "} + #val + " in request";   \
        return crow::response{crow::status::BAD_REQUEST, response.dump()};                   \
    }                                                                                        \
    auto val = type{};                                                                       \
    try {                                                                                    \
        val = json[#val].get<type>();                                                        \
    } catch (const nlohmann::detail::exception& ex) {                                        \
        response["additionalInfo"] = std::string{"Malformed field "} + #val + " in request"; \
        return crow::response{crow::status::BAD_REQUEST, response.dump()};                   \
    }

#define REQUIRED_JSON_VALUE(json, val) REQUIRED_TYPED_JSON_VALUE(json, val, std::string)

#define OPTIONAL_TYPED_JSON_VALUE(json, val, type)        \
    auto val = type{};                                    \
    if (json.contains(#val)) {                            \
        try {                                             \
            val = json[#val].get<std::string>();          \
        } catch (const nlohmann::detail::exception& ex) { \
        }                                                 \
    }

#define OPTIONAL_JSON_VALUE(json, val) OPTIONAL_TYPED_JSON_VALUE(json, val, std::string)

// Module base class - tbd
class module_t
{
public:
    auto load(const fs::path& base_path = "/var/lib/flecs/") //
        -> void;
    auto init() //
        -> void;
    auto deinit() //
        -> void;
    auto save(const fs::path& base_path = "/var/lib/flecs/") const //
        -> void;
    // std::string usage();

protected:
    virtual ~module_t() = default;

private:
    virtual auto do_load(const fs::path& base_path) //
        -> void;
    virtual auto do_init() //
        -> void = 0;
    virtual auto do_deinit() //
        -> void = 0;
    virtual auto do_save(const fs::path& base_path) const //
        -> void;
    // virtual std::string do_usage() = 0;
};

} // namespace FLECS
