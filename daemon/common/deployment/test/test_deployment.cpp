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

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <filesystem>
#include <sstream>

#include "daemon/common/app/app.h"
#include "daemon/common/deployment/deployment.h"

namespace FLECS {
class mock_deployment_t : public deployment_t
{
public:
    MOCK_METHOD(result_t, do_insert_instance, (instance_t instance), (override));
    MOCK_METHOD(result_t, do_create_instance, ((const app_t& app), (instance_t & instance)), (override));
    MOCK_METHOD(result_t, do_delete_instance, (std::string_view instance_id), (override));
    MOCK_METHOD(result_t, do_start_instance, ((const app_t& app), (const instance_t& instance)), (override));
    MOCK_METHOD(result_t, do_ready_instance, (const instance_t& instance), (override));
    MOCK_METHOD(result_t, do_stop_instance, (const instance_t& instance), (override));
    MOCK_METHOD(
        result_t,
        do_create_network,
        ((network_type_t network_type),
         (std::string_view network),
         (std::string_view cidr_subnet),
         (std::string_view gateway),
         (std::string_view parent_adapter)),
        (override));
    MOCK_METHOD(std::optional<network_t>, do_query_network, (std::string_view network), (override));
    MOCK_METHOD(result_t, do_delete_network, (std::string_view network), (override));
    MOCK_METHOD(
        result_t,
        do_connect_network,
        ((std::string_view instance_id), (std::string_view network), (std::string_view ip)),
        (override));
    MOCK_METHOD(
        result_t,
        do_disconnect_network,
        ((std::string_view instance_id), (std::string_view network)),
        (override));
    MOCK_METHOD(
        result_t,
        do_create_volume,
        ((std::string_view instance_id), (std::string_view volume_name)),
        (override));
    MOCK_METHOD(
        result_t,
        do_delete_volume,
        ((std::string_view instance_id), (std::string_view volume_name)),
        (override));
};
} // namespace FLECS

#define G_APP "tech.flecs.test-app"
#define G_CIDR_SUBNET "172.20.0.0/24"
#define G_GATEWAY "172.20.0.1"
#define G_INSTANCE_ID "abcd0123"
#define G_IP "172.20.0.2"
#define G_NAME "Test instance"
#define G_NETWORK_NAME "flecs-network"
#define G_PARENT ""
#define G_VERSION "1.2.3.4-f1"
#define G_VOLUME "flecs-volume"

TEST(deployment, interface)
{
    auto deployment = std::unique_ptr<FLECS::deployment_t>{new FLECS::mock_deployment_t{}};
    auto& test_deployment = static_cast<FLECS::mock_deployment_t&>(*deployment.get());

    using std::operator""s;
    const auto manifest =
        "app: tech.flecs.test-app\n"
        "title: FLECS test app for unit tests"
        "version: 1.2.3.4-f1"
        "author: FLECS Technologies GmbH (info@flecs.tech)"
        "image: flecs/test-app"s;

    const auto app = FLECS::app_t{manifest, FLECS::app_status_e::INSTALLED, FLECS::app_status_e::INSTALLED};

    const auto instance = FLECS::instance_t{
        G_INSTANCE_ID,
        G_APP,
        G_VERSION,
        G_NAME,
        FLECS::instance_status_e::RUNNING,
        FLECS::instance_status_e::RUNNING};

    EXPECT_EQ(test_deployment.instances().count(G_INSTANCE_ID), 0);

    EXPECT_CALL(test_deployment, do_insert_instance(instance)).Times(1);
    deployment->insert_instance(instance);

    EXPECT_EQ(test_deployment.instances().count(G_INSTANCE_ID), 1);

    EXPECT_CALL(test_deployment, do_create_instance).Times(1);
    deployment->create_instance(app, "test instance");

    EXPECT_CALL(test_deployment, do_delete_instance(instance.id())).Times(1);
    deployment->delete_instance(instance.id());

    EXPECT_CALL(test_deployment, do_start_instance(testing::_, instance)).Times(1);
    deployment->start_instance(app, instance.id());

    EXPECT_CALL(test_deployment, do_ready_instance(instance)).Times(1);
    deployment->ready_instance(instance.id());

    EXPECT_CALL(test_deployment, do_stop_instance(instance)).Times(1);
    deployment->stop_instance(instance.id());

    EXPECT_CALL(
        test_deployment,
        do_create_network(FLECS::network_type_t::BRIDGE, G_NETWORK_NAME, G_CIDR_SUBNET, G_GATEWAY, G_PARENT))
        .Times(1);
    deployment->create_network(FLECS::network_type_t::BRIDGE, G_NETWORK_NAME, G_CIDR_SUBNET, G_GATEWAY, G_PARENT);

    EXPECT_CALL(test_deployment, do_query_network(G_NETWORK_NAME)).Times(1);
    deployment->query_network(G_NETWORK_NAME);

    EXPECT_CALL(test_deployment, do_delete_network(G_NETWORK_NAME)).Times(1);
    deployment->delete_network(G_NETWORK_NAME);

    EXPECT_CALL(test_deployment, do_connect_network(G_INSTANCE_ID, G_NETWORK_NAME, G_IP)).Times(1);
    deployment->connect_network(G_INSTANCE_ID, G_NETWORK_NAME, G_IP);

    EXPECT_CALL(test_deployment, do_disconnect_network(G_INSTANCE_ID, G_NETWORK_NAME)).Times(1);
    deployment->disconnect_network(G_INSTANCE_ID, G_NETWORK_NAME);

    EXPECT_CALL(test_deployment, do_create_volume(G_INSTANCE_ID, G_VOLUME)).Times(1);
    deployment->create_volume(G_INSTANCE_ID, G_VOLUME);

    EXPECT_CALL(test_deployment, do_delete_volume(G_INSTANCE_ID, G_VOLUME)).Times(1);
    deployment->delete_volume(G_INSTANCE_ID, G_VOLUME);
}

TEST(deployment, generate_ip_success)
{
    auto deployment = std::unique_ptr<FLECS::deployment_t>{new FLECS::mock_deployment_t{}};

    {
        const auto ip = deployment->generate_instance_ip(G_CIDR_SUBNET, G_GATEWAY);
        EXPECT_EQ(ip, "172.20.0.2");
    }

    auto instance = FLECS::instance_t{
        G_APP,
        G_VERSION,
        G_NAME,
        FLECS::instance_status_e::CREATED,
        FLECS::instance_status_e::CREATED};
    instance.config().networks.emplace_back(FLECS::instance_config_t::network_t{"flecs-network", G_IP});

    deployment->insert_instance(instance);

    {
        const auto ip = deployment->generate_instance_ip(G_CIDR_SUBNET, G_GATEWAY);
        EXPECT_EQ(ip, "172.20.0.3");
    }
}

TEST(deployment, generate_ip_fail)
{
    auto deployment = std::unique_ptr<FLECS::deployment_t>{new FLECS::mock_deployment_t{}};

    // invalid cidr subnet
    {
        const auto ip = deployment->generate_instance_ip("invalid_cidr_subnet", G_GATEWAY);
        EXPECT_TRUE(ip.empty());
    }

    // invalid subnet size
    {
        const auto ip = deployment->generate_instance_ip("172.20.0.0/255", G_GATEWAY);
        EXPECT_TRUE(ip.empty());
    }

    // no free ip left
    {
        const auto ip = deployment->generate_instance_ip("172.20.0.0/32", G_GATEWAY);
        EXPECT_TRUE(ip.empty());
    }
}
