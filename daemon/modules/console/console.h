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

#include <cinttypes>
#include <memory>
#include <string>
#include <string_view>

#include "module_base/module.h"
#include "types/auth_response.h"

namespace flecs {
namespace module {

namespace impl {
class console_t;
} // namespace impl

class console_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    ~console_t() override;

    static constexpr auto base_url() //
        -> std::string_view;

    auto authentication() const noexcept //
        -> const console::auth_response_data_t&;

    auto activate_license(std::string_view session_id) //
        -> result_t;

    auto validate_license(std::string_view session_id) //
        -> result_t;

protected:
    console_t();

    auto do_init() //
        -> void override;
    auto do_deinit() //
        -> void override;

    auto store_authentication(console::auth_response_data_t auth) //
        -> crow::response;
    auto delete_authentication() //
        -> crow::response;

    std::unique_ptr<impl::console_t> _impl;
};

constexpr auto console_t::base_url() //
    -> std::string_view
{
    using std::operator""sv;

#ifndef NDEBUG
    return "https://console-dev.flecs.tech"sv;
#else
    return "https://console.flecs.tech"sv;
#endif // NDEBUG
}

} // namespace module
} // namespace flecs