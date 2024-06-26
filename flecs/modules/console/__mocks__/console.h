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

#include <gmock/gmock.h>

#include "flecs/modules/console/types.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/modules/module_base/module.h"

namespace flecs {
namespace module {

namespace impl {
class console_t
{
public:
    ~console_t() = default;
};
} // namespace impl

class console_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    struct license_activation_result_t {
        std::optional<std::string> error_message;
        std::optional<console::activate_response_data_t> result;
    };
    ~console_t() override = default;

    static constexpr auto base_url() //
        -> std::string_view;

    MOCK_METHOD((const console::auth_response_data_t&), authentication, (), (const, noexcept));
    MOCK_METHOD((license_activation_result_t), activate_license, (std::string license, const std::optional<console::session_id_t>& session_id), ());
    MOCK_METHOD((license_activation_result_t), activate_license_key, (), ());
    MOCK_METHOD((result_t), validate_license, (std::string_view session_id), ());
    MOCK_METHOD((std::string), download_manifest, (std::string, std::string, std::string), ());
    MOCK_METHOD(
        (std::optional<flecs::console::download_token_t>),
        acquire_download_token,
        (std::string, std::string, std::string));

protected:
    console_t() = default;

    MOCK_METHOD((void), do_init, (), (override));
    MOCK_METHOD((void), do_deinit, (), (override));

    MOCK_METHOD((crow::response), store_authentication, (console::auth_response_data_t auth), ());
    MOCK_METHOD((crow::response), delete_authentication, (), ());

    std::unique_ptr<impl::console_t> _impl;
};

constexpr auto console_t::base_url() //
    -> std::string_view
{
    using std::operator""sv;

#if defined FLECS_UNIT_TEST
    return "http://127.0.0.1:18952"sv;
#elif defined NDEBUG
    return "https://console.flecs.tech"sv;
#else
    return "https://console-dev.flecs.tech"sv;
#endif // FLECS_UNIT_TEST
}

} // namespace module
} // namespace flecs
