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

#include <sstream>

#include "daemon/common/app/app.h"
#include "daemon/common/deployment/deployment.h"
#include "util/fs/fs.h"

namespace FLECS {
class mock_deployment_t : public deployment_t
{
public:
    MOCK_METHOD(std::string_view, do_deployment_id, (), (const, noexcept, override));
    MOCK_METHOD(result_t, do_insert_instance, (instance_t instance), (override));
    MOCK_METHOD(result_t, do_create_instance, ((const app_t& app), (instance_t & instance)), (override));
    MOCK_METHOD(result_t, do_delete_instance, (std::string_view instance_id), (override));
    MOCK_METHOD(result_t, do_start_instance, ((instance_t & instance)), (override));
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
    MOCK_METHOD(bool, do_is_instance_running, (const instance_t& instance), (const, override));
    MOCK_METHOD(std::optional<network_t>, do_query_network, (std::string_view network), (override));
    MOCK_METHOD(result_t, do_delete_network, (std::string_view network), (override));
    MOCK_METHOD(
        result_t,
        do_connect_network,
        ((std::string_view instance_id), (std::string_view network), (std::string_view ip)),
        (override));
    MOCK_METHOD(
        result_t, do_disconnect_network, ((std::string_view instance_id), (std::string_view network)), (override));
    MOCK_METHOD(
        result_t, do_create_volume, ((std::string_view instance_id), (std::string_view volume_name)), (override));
    MOCK_METHOD(
        result_t,
        do_import_volume,
        ((const instance_t& instance), (std::string_view volume_name), (FLECS::fs::path dest_dir)),
        (override));
    MOCK_METHOD(
        result_t,
        do_export_volume,
        ((const instance_t& instance), (std::string_view volume_name), (FLECS::fs::path dest_dir)),
        (override));
    MOCK_METHOD(
        result_t, do_delete_volume, ((std::string_view instance_id), (std::string_view volume_name)), (override));
    MOCK_METHOD(
        result_t, do_copy_file_from_image, ((std::string_view image), (fs::path file), (fs::path dest)), (override));
    MOCK_METHOD(
        result_t,
        do_copy_file_to_instance,
        ((std::string_view instance_id), (fs::path file), (fs::path dest)),
        (override));
    MOCK_METHOD(
        result_t,
        do_copy_file_from_instance,
        ((std::string_view instance_id), (fs::path file), (fs::path dest)),
        (override));
    MOCK_METHOD(std::string_view, do_default_network_name, (), (const, override));
    MOCK_METHOD(network_type_t, do_default_network_type, (), (const, override));
    MOCK_METHOD(std::string_view, do_default_network_cidr_subnet, (), (const, override));
    MOCK_METHOD(std::string_view, do_default_network_gateway, (), (const, override));
};
} // namespace FLECS

using std::operator""s;

#define G_APP "tech.flecs.test-app"
#define G_CIDR_SUBNET "172.20.0.0/24"
#define G_GATEWAY "172.20.0.1"
#define G_IMAGE "flecs/test-app"
#define G_INSTANCE_ID_1 "abcd0123"
#define G_INSTANCE_ID_2 "0123abcd"
#define G_IP "172.20.0.2"
#define G_INSTANCE_NAME_1 "Test instance 1"
#define G_INSTANCE_NAME_2 "Test instance 2"
#define G_NETWORK_NAME "flecs-network"
#define G_PARENT ""
#define G_VERSION_1 "1.2.3.4-f1"
#define G_VERSION_2 "2.3.4.5-f1"
#define G_VOLUME "flecs-volume"
#define G_FILE_LOCAL "/some/local/file"
#define G_FILE_CONTAINER "/some/other/container/file"

#define G_MANIFEST_1                                      \
    "app: tech.flecs.test-app\n"                          \
    "title: FLECS test app for unit tests\n"              \
    "version: " G_VERSION_1                               \
    "\n"                                                  \
    "author: FLECS Technologies GmbH (info@flecs.tech)\n" \
    "image: " G_IMAGE "\n"s

#define G_MANIFEST_2                                      \
    "app: tech.flecs.test-app\n"                          \
    "title: FLECS test app for unit tests\n"              \
    "version: " G_VERSION_2                               \
    "\n"                                                  \
    "author: FLECS Technologies GmbH (info@flecs.tech)\n" \
    "image: " G_IMAGE "\n"s

static const auto app_1 = FLECS::app_t{G_MANIFEST_1, FLECS::app_status_e::INSTALLED, FLECS::app_status_e::INSTALLED};
static const auto app_2 = FLECS::app_t{G_MANIFEST_2, FLECS::app_status_e::INSTALLED, FLECS::app_status_e::INSTALLED};

TEST(deployment, interface)
{
    auto deployment = std::unique_ptr<FLECS::deployment_t>{new FLECS::mock_deployment_t{}};
    auto& test_deployment = static_cast<FLECS::mock_deployment_t&>(*deployment.get());
    const auto& test_deployment_c = static_cast<const FLECS::mock_deployment_t&>(test_deployment);

    auto instance_1 = FLECS::instance_t{
        G_INSTANCE_ID_1,
        &app_1,
        G_INSTANCE_NAME_1,
        FLECS::instance_status_e::CREATED,
        FLECS::instance_status_e::RUNNING};

    auto instance_2 = FLECS::instance_t{
        G_INSTANCE_ID_2,
        &app_2,
        G_INSTANCE_NAME_2,
        FLECS::instance_status_e::CREATED,
        FLECS::instance_status_e::RUNNING};

    EXPECT_CALL(test_deployment, do_deployment_id()).Times(1);
    test_deployment.deployment_id();

    // deployment should be empty initially
    ASSERT_TRUE(test_deployment.instances().empty());
    ASSERT_EQ(test_deployment.instances().count(G_INSTANCE_ID_1), 0);
    ASSERT_FALSE(test_deployment.has_instance(G_INSTANCE_ID_1));
    ASSERT_FALSE(test_deployment.is_instance_runnable(G_INSTANCE_ID_1));
    EXPECT_CALL(test_deployment, do_is_instance_running(instance_1)).Times(0);
    deployment->is_instance_running(G_INSTANCE_ID_1);

    // insert instance with of app_1 with ID_1
    EXPECT_CALL(test_deployment, do_insert_instance(instance_1)).Times(1);
    deployment->insert_instance(instance_1);

    // deployment should now contain ID_1
    ASSERT_FALSE(test_deployment_c.instances().empty());
    ASSERT_EQ(test_deployment_c.instances().count(G_INSTANCE_ID_1), 1);
    ASSERT_TRUE(test_deployment_c.has_instance(G_INSTANCE_ID_1));
    ASSERT_TRUE(test_deployment.is_instance_runnable(G_INSTANCE_ID_1));
    EXPECT_CALL(test_deployment, do_is_instance_running(instance_1)).Times(1);
    deployment->is_instance_running(G_INSTANCE_ID_1);

    const auto ids_1 = test_deployment.instance_ids(app_1);
    EXPECT_EQ(ids_1.size(), 1);
    EXPECT_EQ(ids_1[0], G_INSTANCE_ID_1);

    // create instance of app_1 with random instance id
    EXPECT_CALL(test_deployment, do_create_instance).Times(1);
    deployment->create_instance(app_1, "test instance_1");

    const auto ids_2 = test_deployment.instance_ids(app_1);
    EXPECT_EQ(ids_2.size(), 2);

    const auto ids_3 = test_deployment.instance_ids(app_1.app(), app_1.version());
    EXPECT_EQ(ids_3.size(), 2);

    // insert instance of app_2 with ID_2
    EXPECT_CALL(test_deployment, do_insert_instance(instance_2)).Times(1);
    deployment->insert_instance(instance_2);

    // deployment should now contain ID_2
    ASSERT_EQ(test_deployment_c.instances().count(G_INSTANCE_ID_2), 1);
    ASSERT_TRUE(test_deployment_c.has_instance(G_INSTANCE_ID_2));

    const auto ids_4 = test_deployment.instance_ids(app_1.app());
    EXPECT_EQ(ids_4.size(), 3);

    const auto ids_5 = test_deployment.instance_ids(app_1.app(), app_1.version());
    EXPECT_EQ(ids_5.size(), 2);

    EXPECT_CALL(test_deployment, do_start_instance(instance_1)).Times(1);
    deployment->start_instance(instance_1.id());

    EXPECT_CALL(test_deployment, do_ready_instance(instance_1)).Times(1);
    deployment->ready_instance(instance_1.id());

    EXPECT_CALL(test_deployment, do_stop_instance(instance_1)).Times(1);
    deployment->stop_instance(instance_1.id());

    EXPECT_CALL(test_deployment, do_delete_instance(instance_1.id())).Times(1);
    deployment->delete_instance(instance_1.id());

    EXPECT_EQ(test_deployment.instances().count(G_INSTANCE_ID_1), 0);

    EXPECT_CALL(
        test_deployment,
        do_create_network(FLECS::network_type_t::BRIDGE, G_NETWORK_NAME, G_CIDR_SUBNET, G_GATEWAY, G_PARENT))
        .Times(1);
    deployment->create_network(FLECS::network_type_t::BRIDGE, G_NETWORK_NAME, G_CIDR_SUBNET, G_GATEWAY, G_PARENT);

    EXPECT_CALL(test_deployment, do_query_network(G_NETWORK_NAME)).Times(1);
    deployment->query_network(G_NETWORK_NAME);

    EXPECT_CALL(test_deployment, do_delete_network(G_NETWORK_NAME)).Times(1);
    deployment->delete_network(G_NETWORK_NAME);

    EXPECT_CALL(test_deployment, do_connect_network(G_INSTANCE_ID_1, G_NETWORK_NAME, G_IP)).Times(1);
    deployment->connect_network(G_INSTANCE_ID_1, G_NETWORK_NAME, G_IP);

    EXPECT_CALL(test_deployment, do_disconnect_network(G_INSTANCE_ID_1, G_NETWORK_NAME)).Times(1);
    deployment->disconnect_network(G_INSTANCE_ID_1, G_NETWORK_NAME);

    EXPECT_CALL(test_deployment, do_create_volume(G_INSTANCE_ID_1, G_VOLUME)).Times(1);
    deployment->create_volume(G_INSTANCE_ID_1, G_VOLUME);

    EXPECT_CALL(test_deployment, do_delete_volume(G_INSTANCE_ID_1, G_VOLUME)).Times(1);
    deployment->delete_volume(G_INSTANCE_ID_1, G_VOLUME);

    EXPECT_CALL(
        test_deployment,
        do_copy_file_from_image(G_IMAGE, FLECS::fs::path{G_FILE_CONTAINER}, FLECS::fs::path{G_FILE_LOCAL}))
        .Times(1);
    deployment->copy_file_from_image(G_IMAGE, G_FILE_CONTAINER, G_FILE_LOCAL);

    EXPECT_CALL(
        test_deployment,
        do_copy_file_to_instance(G_INSTANCE_ID_1, FLECS::fs::path{G_FILE_LOCAL}, FLECS::fs::path{G_FILE_CONTAINER}))
        .Times(1);
    deployment->copy_file_to_instance(G_INSTANCE_ID_1, G_FILE_LOCAL, G_FILE_CONTAINER);

    EXPECT_CALL(
        test_deployment,
        do_copy_file_from_instance(G_INSTANCE_ID_1, FLECS::fs::path{G_FILE_CONTAINER}, FLECS::fs::path{G_FILE_LOCAL}))
        .Times(1);
    deployment->copy_file_from_instance(G_INSTANCE_ID_1, G_FILE_CONTAINER, G_FILE_LOCAL);

    EXPECT_CALL(test_deployment, do_default_network_name()).Times(1);
    deployment->default_network_name();

    EXPECT_CALL(test_deployment, do_default_network_type()).Times(1);
    deployment->default_network_type();

    EXPECT_CALL(test_deployment, do_default_network_cidr_subnet()).Times(1);
    deployment->default_network_cidr_subnet();

    EXPECT_CALL(test_deployment, do_default_network_gateway()).Times(1);
    deployment->default_network_gateway();
}

TEST(deployment, load_save)
{
    auto save_deployment = std::unique_ptr<FLECS::deployment_t>{new FLECS::mock_deployment_t{}};
    auto& save_uut = static_cast<FLECS::mock_deployment_t&>(*save_deployment.get());

    auto instance_1 = FLECS::instance_t{
        G_INSTANCE_ID_1,
        &app_1,
        G_INSTANCE_NAME_1,
        FLECS::instance_status_e::CREATED,
        FLECS::instance_status_e::RUNNING};

    auto instance_2 = FLECS::instance_t{
        G_INSTANCE_ID_2,
        &app_2,
        G_INSTANCE_NAME_2,
        FLECS::instance_status_e::CREATED,
        FLECS::instance_status_e::RUNNING};

    save_deployment->insert_instance(instance_1);
    save_deployment->insert_instance(instance_2);
    EXPECT_CALL(save_uut, do_deployment_id()).Times(1).WillOnce(testing::Return("test"));
    save_deployment->save(".");

    auto load_deployment = std::unique_ptr<FLECS::deployment_t>{new FLECS::mock_deployment_t{}};
    auto& load_uut = static_cast<FLECS::mock_deployment_t&>(*load_deployment.get());

    EXPECT_CALL(load_uut, do_deployment_id()).Times(1).WillOnce(testing::Return("test"));
    load_deployment->load(".");

    ASSERT_EQ(load_deployment->instances().size(), 2);
    ASSERT_EQ(load_deployment->instances().at(G_INSTANCE_ID_1), instance_1);
    ASSERT_EQ(load_deployment->instances().at(G_INSTANCE_ID_2), instance_2);
}

TEST(deployment, generate_ip_success)
{
    auto deployment = std::unique_ptr<FLECS::deployment_t>{new FLECS::mock_deployment_t{}};

    {
        const auto ip = deployment->generate_instance_ip(G_CIDR_SUBNET, G_GATEWAY);
        EXPECT_EQ(ip, "172.20.0.2");
    }

    auto instance = FLECS::instance_t{
        &app_1,
        G_INSTANCE_NAME_1,
        FLECS::instance_status_e::CREATED,
        FLECS::instance_status_e::CREATED};
    instance.networks().emplace_back(
        FLECS::instance_t::network_t{.network_name = "flecs-network", .mac_address = {}, .ip_address = G_IP});

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
