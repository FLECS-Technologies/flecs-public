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

#include "module_base/module.h"

namespace FLECS {

class module_marketplace_t FLECS_FINAL_UNLESS_TESTED : public module_t
{
    friend class module_factory_t;

public:
    auto& user() const noexcept { return _user; }
    auto& token() const noexcept { return _token; }

protected:
    module_marketplace_t();

    auto do_init() //
        -> void override;
    auto do_deinit() //
        -> void override;

    auto login(std::string user, std::string token) //
        -> crow::response;
    auto logout(std::string_view user) //
        -> crow::response;

    std::string _user;
    std::string _token;
};

} // namespace FLECS
