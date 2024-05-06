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

#include "flecs/modules/console/__mocks__/console.h"
#include "flecs/modules/device/device.h"
#include "flecs/modules/factory/factory.h"

class test_module_device_t : public flecs::module::device_t
{
public:
    test_module_device_t() { flecs::module::register_module_t<flecs::module::console_t>("console"); }

    ~test_module_device_t() { flecs::module::unregister_module_t("console"); }
};

const auto session_id_regex = std::regex{"[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}"};

TEST(device, session_id)
{
    auto session_id = flecs::console::session_id_t{"11157500-6255-4d1d-ae0b-7c83f0f47eee", 1289};
    {
        std::filesystem::remove_all("./device");

        auto uut = test_module_device_t{};
        uut.init();

        /* No .session_id file present -- loading should fail */
        auto [res, message] = uut.load(".");
        ASSERT_EQ(res, -1);

        ASSERT_FALSE(uut.session_id().has_value());

        /* Session id should be saved correctly */
        std::tie(res, message) = uut.save_session_id(session_id, ".");
        if (res != 0) {
            std::cerr << message << "\n";
        }
        ASSERT_EQ(res, 0);

        /* Should successfully create .session_id */
        std::tie(res, message) = uut.save(".");
        if (res != 0) {
            std::cerr << message << "\n";
        }
        ASSERT_EQ(res, 0);

        uut.deinit();
    }
    {
        auto uut = test_module_device_t{};
        uut.init();

        /* .session_id created in previous test case -- loading should succeed */
        auto [res, message] = uut.load(".");
        if (res != 0) {
            std::cerr << message << "\n";
        }
        ASSERT_EQ(res, 0);
        ASSERT_EQ(session_id, uut.session_id());

        /* Should successfully overwrite .session_id */
        std::tie(res, message) = uut.save(".");
        if (res != 0) {
            std::cerr << message << "\n";
        }
        ASSERT_EQ(res, 0);

        uut.deinit();
    }
    {
        {
            auto f = std::ofstream{"./device/.session_id", std::ios::trunc};
            f << "invalid-session-id";
        }

        auto uut = test_module_device_t{};
        uut.init();

        /* .session_id contains garbage -- loading should fail */
        const auto [res, message] = uut.load(".");
        ASSERT_EQ(res, -1);
        /* new, random session_id should be generated */
        ASSERT_NE(session_id, uut.session_id());

        uut.save(".");
        uut.deinit();
    }
    {
        auto uut = test_module_device_t{};
        uut.init();
        uut.load(".");

        /* Saving under /proc should fail */
        const auto [res, message] = uut.save_session_id(session_id, "/proc");
        ASSERT_EQ(res, -1);
        uut.deinit();
    }
}

TEST(device, activate_license)
{
    auto uut = test_module_device_t{};
    uut.init();

    auto mock_console =
        std::dynamic_pointer_cast<flecs::module::console_t>(flecs::api::query_module("console"));

    EXPECT_CALL(*mock_console.get(), activate_license_key());

    uut.activate_license();

    uut.deinit();
}

TEST(device, validate_license)
{
    auto uut = test_module_device_t{};
    uut.init();
    auto session_id = flecs::console::session_id_t{"11157500-6255-4d1d-ae0b-7c83f0f47eee", 1289};

    auto [res, message] = uut.save_session_id(session_id, ".");
    if (res != 0) {
        std::cerr << message << "\n";
    }
    ASSERT_EQ(res, 0);

    auto mock_console =
        std::dynamic_pointer_cast<flecs::module::console_t>(flecs::api::query_module("console"));

    ASSERT_TRUE(uut.session_id().has_value());
    EXPECT_CALL(*mock_console.get(), validate_license(uut.session_id().value().id()));

    uut.validate_license();

    uut.deinit();
}
