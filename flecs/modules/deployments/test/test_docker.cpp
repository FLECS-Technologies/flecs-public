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

#include <gtest/gtest.h>

#include <algorithm>
#include <string>
#include <type_traits>

#include "flecs/modules/deployments/types/deployment_docker.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/modules/system/system.h"
#include "flecs/util/process/process.h"

class deployment_docker_test : public testing::Test
{
protected:
    static void SetUpTestSuite()
    {
        flecs::module::register_module_t<flecs::module::system_t>("system");
        auto sys = std::dynamic_pointer_cast<flecs::module::system_t>(flecs::api::query_module("system"));

        auto adapters = sys->get_network_adapters();
        const auto wired_adapter = std::find_if(
            adapters.cbegin(),
            adapters.cend(),
            [](std::remove_cv_t<decltype(adapters)>::const_reference a) {
                return a.second.net_type == NetType::Wired;
            });
        if (wired_adapter != adapters.cend()) {
            _parent_adapter = wired_adapter->first;
        } else {
            _parent_adapter = _parent_adapter;

            auto p = flecs::process_t{};
            p.spawnp("ip", "link", "add", _parent_adapter, "type", "dummy");
            p.wait(false, true);
            ASSERT_EQ(p.exit_code(), 0);
            _eth_dummy_created = true;
        }
    }

    static void TearDownTestSuite()
    {
        flecs::module::unregister_module_t("system");

        if (_eth_dummy_created) {
            auto p = flecs::process_t{};
            p.spawnp("ip", "link", "delete", _parent_adapter);
            p.wait(false, true);
            ASSERT_EQ(p.exit_code(), 0);
        }
    }

    static bool _eth_dummy_created;
    static std::string _parent_adapter;
    flecs::deployments::docker_t uut;
};

bool deployment_docker_test::_eth_dummy_created = {};
std::string deployment_docker_test::_parent_adapter = {};

TEST_F(deployment_docker_test, create_network)
{
    uut.delete_network("flecs-unit-test");
    {
        const auto networks = uut.networks();
        ASSERT_TRUE(
            std::find_if(
                networks.cbegin(),
                networks.cend(),
                [](std::remove_cv_t<decltype(networks)>::const_reference n) {
                    return n.name == "flecs-unit-test";
                }) == networks.cend());
    }
    {
        const auto [res, message] = uut.create_network(
            flecs::network_type_e::IPVLAN_L2,
            "flecs-unit-test",
            "10.0.0.0/24",
            "10.0.0.1",
            _parent_adapter);
        ASSERT_EQ(res, 0);
    }
    {
        const auto networks = uut.networks();
        ASSERT_FALSE(
            std::find_if(
                networks.cbegin(),
                networks.cend(),
                [](std::remove_cv_t<decltype(networks)>::const_reference n) {
                    return n.name == "flecs-unit-test";
                }) == networks.cend());
    }
}

TEST_F(deployment_docker_test, query_network)
{
    const auto network = uut.query_network("flecs-unit-test");

    ASSERT_TRUE(network.has_value());
    ASSERT_EQ(network->type, flecs::network_type_e::IPVLAN_L2);
    ASSERT_EQ(network->name, "flecs-unit-test");
    ASSERT_EQ(network->cidr_subnet, "10.0.0.0/24");
    ASSERT_EQ(network->gateway, "10.0.0.1");
    ASSERT_EQ(network->parent, _parent_adapter);
}

TEST_F(deployment_docker_test, delete_network)
{
    {
        const auto networks = uut.networks();
        ASSERT_FALSE(
            std::find_if(
                networks.cbegin(),
                networks.cend(),
                [](std::remove_cv_t<decltype(networks)>::const_reference n) {
                    return n.name == "flecs-unit-test";
                }) == networks.cend());
    }
    {
        const auto [res, message] = uut.delete_network("flecs-unit-test");
        ASSERT_EQ(res, 0);
    }
    {
        const auto networks = uut.networks();
        ASSERT_TRUE(
            std::find_if(
                networks.cbegin(),
                networks.cend(),
                [](std::remove_cv_t<decltype(networks)>::const_reference n) {
                    return n.name == "flecs-unit-test";
                }) == networks.cend());
    }
}
