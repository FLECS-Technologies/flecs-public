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

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <regex>

#include "daemon/modules/console/__mocks__/console.h"
#include "daemon/modules/device/device.h"
#include "daemon/modules/factory/factory.h"

class test_module_device_t : public flecs::module::device_t
{
public:
    test_module_device_t()
    {
        flecs::module::register_module_t<flecs::module::console_t>("console");
    }

    ~test_module_device_t() { flecs::module::unregister_module_t("console"); }
};

const auto session_id_regex = std::regex{"[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}"};

TEST(device, session_id)
{
    auto uut = test_module_device_t{};
    auto session_id = std::string{};
    {
        uut.init();
        uut.load(".");

        session_id = uut.session_id();
        ASSERT_TRUE(std::regex_match(session_id, session_id_regex));

        uut.deinit();
        uut.save(".");
    }
    {
        uut.init();
        uut.load(".");

        ASSERT_EQ(session_id, uut.session_id());

        uut.deinit();
        uut.save(".");
    }
}

TEST(device, activate_license)
{
    auto uut = test_module_device_t{};
    uut.init();
    const auto session_id = uut.session_id();

    auto mock_console =
        std::dynamic_pointer_cast<flecs::module::console_t>(flecs::api::query_module("console"));

    EXPECT_CALL(*mock_console.get(), activate_license(session_id));

    uut.activate_license();

    uut.deinit();
}

TEST(device, validate_license)
{
    auto uut = test_module_device_t{};
    uut.init();
    const auto session_id = uut.session_id();

    auto mock_console =
        std::dynamic_pointer_cast<flecs::module::console_t>(flecs::api::query_module("console"));

    EXPECT_CALL(*mock_console.get(), validate_license(session_id));

    uut.validate_license();

    uut.deinit();
}
