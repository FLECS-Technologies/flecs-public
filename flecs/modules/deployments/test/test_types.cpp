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

#include <sstream>
#include <string_view>

#include "flecs/common/app/manifest/manifest.h"
#include "flecs/modules/apps/types/app.h"
#include "flecs/modules/apps/types/app_key.h"
#include "flecs/modules/deployments/types.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/modules/floxy/__mocks__/floxy.h"
#include "flecs/util/fs/fs.h"

namespace flecs {
namespace deployments {
class mock_deployment_t : public deployment_t
{
public:
    MOCK_METHOD(std::string_view, do_deployment_id, (), (const, noexcept, override));
    MOCK_METHOD(
        result_t,
        do_download_app,
        (std::shared_ptr<apps::app_t> app, std::optional<Token> token),
        (override));
    MOCK_METHOD(
        std::optional<std::size_t>,
        do_determine_app_size,
        (std::shared_ptr<const apps::app_t> app),
        (const, override));
    MOCK_METHOD((result_t), do_delete_app, (std::shared_ptr<apps::app_t> app), (override));
    MOCK_METHOD(result_t, do_create_instance, (std::shared_ptr<instances::instance_t> instance), (override));
    MOCK_METHOD(result_t, do_delete_instance, (std::shared_ptr<instances::instance_t> instance), (override));
    MOCK_METHOD(result_t, do_start_instance, (std::shared_ptr<instances::instance_t> instance), (override));
    MOCK_METHOD(result_t, do_ready_instance, (std::shared_ptr<instances::instance_t> instance), (override));
    MOCK_METHOD(result_t, do_stop_instance, (std::shared_ptr<instances::instance_t> instance), (override));
    MOCK_METHOD(
        result_t,
        do_export_instance,
        (std::shared_ptr<instances::instance_t> instance, fs::path dest_dir),
        (const, override));
    MOCK_METHOD(
        result_t,
        do_import_instance,
        (std::shared_ptr<instances::instance_t> instance, fs::path base_dir),
        (override));
    MOCK_METHOD(
        result_t,
        do_create_network,
        ((network_type_e network_type),
         (std::string network_name),
         (std::string cidr_subnet),
         (std::string gateway),
         (std::string parent_adapter)),
        (override));
    MOCK_METHOD(
        bool, do_is_instance_running, (std::shared_ptr<instances::instance_t> instance), (const, override));
    MOCK_METHOD(std::vector<network_t>, do_networks, (), (const, override));
    MOCK_METHOD(std::optional<network_t>, do_query_network, (std::string_view network), (const, override));
    MOCK_METHOD(result_t, do_delete_network, (std::string_view network), (override));
    MOCK_METHOD(
        result_t,
        do_connect_network,
        ((std::shared_ptr<instances::instance_t> instance),
         (std::string_view network),
         (std::string_view ip)),
        (override));
    MOCK_METHOD(
        result_t,
        do_disconnect_network,
        ((std::shared_ptr<instances::instance_t> instance), (std::string_view network)),
        (override));
    MOCK_METHOD(
        result_t,
        do_create_volume,
        ((std::shared_ptr<instances::instance_t> instance), (std::string_view volume_name)),
        (override));
    MOCK_METHOD(
        result_t,
        do_import_volume,
        ((std::shared_ptr<instances::instance_t> instance), (volume_t & volume), (flecs::fs::path dest_dir)),
        (override));
    MOCK_METHOD(
        result_t,
        do_export_volume,
        ((std::shared_ptr<instances::instance_t> instance),
         (const volume_t& volume),
         (flecs::fs::path dest_dir)),
        (const, override));
    MOCK_METHOD(
        result_t,
        do_delete_volume,
        ((std::shared_ptr<instances::instance_t> instance), (std::string_view volume_name)),
        (override));
    MOCK_METHOD(
        result_t,
        do_copy_file_from_image,
        ((std::string_view image), (fs::path file), (fs::path dest)),
        (override));
    MOCK_METHOD(
        result_t,
        do_copy_file_to_instance,
        ((std::shared_ptr<instances::instance_t> instance), (fs::path file), (fs::path dest)),
        (override));
    MOCK_METHOD(
        result_t,
        do_copy_file_from_instance,
        ((std::shared_ptr<instances::instance_t> instance), (fs::path file), (fs::path dest)),
        (const, override));
    MOCK_METHOD(std::string_view, do_default_network_name, (), (const, override));
    MOCK_METHOD(network_type_e, do_default_network_type, (), (const, override));
    MOCK_METHOD(std::string_view, do_default_network_cidr_subnet, (), (const, override));
    MOCK_METHOD(std::string_view, do_default_network_gateway, (), (const, override));
    mock_deployment_t() { flecs::module::register_module_t<flecs::module::floxy_t>("floxy"); }
    ~mock_deployment_t() { flecs::module::unregister_module_t("floxy"); }
};
} // namespace deployments
} // namespace flecs

using std::operator""s;
using std::operator""sv;
using namespace testing;

#define G_APP "tech.flecs.test-app"
#define G_CIDR_SUBNET "172.20.0.0/24"
#define G_GATEWAY "172.20.0.1"
#define G_IMAGE "flecs/test-app"
#define G_INSTANCE_ID_1 flecs::instances::id_t(2882339107U)
#define G_INSTANCE_ID_2 flecs::instances::id_t(19114957U)
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

static const auto json_manifest_1 = R"-({"app":"tech.flecs.test-app",)-"
                                    R"-("version":"1.2.3.4-f1",)-"
                                    R"-("image":"flecs/test-app",)-";

static const auto json_manifest_2 = R"-({"app":"tech.flecs.test-app",)-"
                                    R"-("version":"2.3.4.5-f1",)-"
                                    R"-("image":"flecs/test-app",)-";

static const auto manifest_1 =
    std::make_shared<flecs::app_manifest_t>(flecs::app_manifest_t::from_json_string(json_manifest_1));
static const auto manifest_2 =
    std::make_shared<flecs::app_manifest_t>(flecs::app_manifest_t::from_json_string(json_manifest_2));

static const auto app_1 =
    std::make_shared<flecs::apps::app_t>(flecs::apps::key_t{G_APP, G_VERSION_1}, manifest_1);
static const auto app_2 =
    std::make_shared<flecs::apps::app_t>(flecs::apps::key_t{G_APP, G_VERSION_2}, manifest_2);

TEST(deployment, interface)
{
    auto deployment =
        std::unique_ptr<flecs::deployments::deployment_t>{new flecs::deployments::mock_deployment_t{}};
    auto& test_deployment = static_cast<flecs::deployments::mock_deployment_t&>(*deployment.get());
    const auto& test_deployment_c =
        static_cast<const flecs::deployments::mock_deployment_t&>(test_deployment);

    auto instance_1 = flecs::instances::instance_t{G_INSTANCE_ID_1, app_1, G_INSTANCE_NAME_1};
    instance_1.status(flecs::instances::status_e::Created);
    instance_1.desired(flecs::instances::status_e::Running);

    auto instance_2 = flecs::instances::instance_t{G_INSTANCE_ID_2, app_2, G_INSTANCE_NAME_2};
    instance_2.status(flecs::instances::status_e::Created);
    instance_2.desired(flecs::instances::status_e::Running);

    // mock deployment id
    EXPECT_CALL(test_deployment, do_deployment_id()).Times(1).WillOnce(Return("test-deployment"sv));
    ASSERT_EQ(test_deployment.deployment_id(), "test-deployment");

    // deployment should be initially empty
    ASSERT_TRUE(test_deployment.instance_ids().empty());
    ASSERT_FALSE(test_deployment_c.has_instance(G_INSTANCE_ID_1));
    ASSERT_FALSE(test_deployment.query_instance(G_INSTANCE_ID_1));

    // insert instance with of app_1 with ID_1
    deployment->insert_instance(instance_1);
    {
        // deployment should now contain ID_1
        ASSERT_FALSE(test_deployment_c.instance_ids().empty());
        ASSERT_TRUE(test_deployment_c.has_instance(G_INSTANCE_ID_1));
        // instance should be runnable (i.e. exists and is 'Created')
        auto p = test_deployment_c.query_instance(G_INSTANCE_ID_1);
        ASSERT_TRUE(p);
        ASSERT_TRUE(test_deployment.is_instance_runnable(p));
        // instance should not be running
        EXPECT_CALL(test_deployment, do_is_instance_running(p)).Times(1).WillOnce(Return(false));
        ASSERT_FALSE(deployment->is_instance_running(p));
    }

    // create instance of app_1 with random instance id
    EXPECT_CALL(test_deployment, do_create_instance).Times(1);
    deployment->create_instance(app_1, "test instance_1");
    {
        // deployment should now contain 2 IDs
        const auto ids = test_deployment.instance_ids(app_1->key().name(), app_1->key().version());
        EXPECT_EQ(ids.size(), 2);
    }

    // insert instance of app_2 with ID_2
    deployment->insert_instance(instance_2);
    {
        // deployment should now contain ID_1
        ASSERT_TRUE(test_deployment_c.has_instance(G_INSTANCE_ID_2));
    }

    {
        // assert content of deployment through different interfaces
        const auto& key_1 = app_1->key();
        const auto& key_2 = app_2->key();

        ASSERT_EQ(test_deployment.instance_ids().size(), 3);
        ASSERT_EQ(test_deployment.instance_ids(key_1.name()).size(), 3);

        ASSERT_EQ(test_deployment.instance_ids(key_1.name(), key_1.version()).size(), 2);
        ASSERT_EQ(test_deployment.instance_ids(key_2.name(), key_2.version()).size(), 1);
    }

    // perform actions on instance_1
    {
        auto p = test_deployment.query_instance(G_INSTANCE_ID_1);
        EXPECT_CALL(test_deployment, do_start_instance(p)).Times(1);
        deployment->start_instance(p);

        EXPECT_CALL(test_deployment, do_ready_instance(p)).Times(1);
        deployment->ready_instance(p);

        EXPECT_CALL(test_deployment, do_stop_instance(p)).Times(1);
        deployment->stop_instance(p);

        EXPECT_CALL(test_deployment, do_delete_instance(p)).Times(1);
        deployment->delete_instance(p);

        ASSERT_FALSE(test_deployment.has_instance(G_INSTANCE_ID_1));
    }

    {
        // interact with networking interface
        EXPECT_CALL(
            test_deployment,
            do_create_network(
                flecs::network_type_e::Bridge,
                G_NETWORK_NAME,
                G_CIDR_SUBNET,
                G_GATEWAY,
                G_PARENT))
            .Times(1);
        deployment->create_network(
            flecs::network_type_e::Bridge,
            G_NETWORK_NAME,
            G_CIDR_SUBNET,
            G_GATEWAY,
            G_PARENT);

        EXPECT_CALL(test_deployment, do_networks()).Times(1);
        deployment->networks();

        EXPECT_CALL(test_deployment, do_query_network(G_NETWORK_NAME)).Times(1);
        deployment->query_network(G_NETWORK_NAME);

        EXPECT_CALL(test_deployment, do_delete_network(G_NETWORK_NAME)).Times(1);
        deployment->delete_network(G_NETWORK_NAME);

        EXPECT_CALL(test_deployment, do_default_network_name()).Times(1).WillOnce(Return("test-network"));
        ASSERT_EQ(deployment->default_network_name(), "test-network");

        EXPECT_CALL(test_deployment, do_default_network_type()).Times(1);
        deployment->default_network_type();

        EXPECT_CALL(test_deployment, do_default_network_cidr_subnet()).Times(1);
        deployment->default_network_cidr_subnet();

        EXPECT_CALL(test_deployment, do_default_network_gateway()).Times(1);
        deployment->default_network_gateway();
    }

    {
        // connect and disconnect network to/from instance_1
        auto p = deployment->query_instance(G_INSTANCE_ID_1);
        EXPECT_CALL(test_deployment, do_connect_network(p, G_NETWORK_NAME, G_IP)).Times(1);
        deployment->connect_network(p, G_NETWORK_NAME, G_IP);

        EXPECT_CALL(test_deployment, do_disconnect_network(p, G_NETWORK_NAME)).Times(1);
        deployment->disconnect_network(p, G_NETWORK_NAME);
    }

    {
        // create and delete volumes for instance_1
        auto p = deployment->query_instance(G_INSTANCE_ID_1);

        EXPECT_CALL(test_deployment, do_create_volume(p, G_VOLUME)).Times(1);
        deployment->create_volume(p, G_VOLUME);

        EXPECT_CALL(test_deployment, do_delete_volume(p, G_VOLUME)).Times(1);
        deployment->delete_volume(p, G_VOLUME);
    }

    {
        // copy files from app/instance
        auto p = deployment->query_instance(G_INSTANCE_ID_2);

        EXPECT_CALL(
            test_deployment,
            do_copy_file_from_image(
                G_IMAGE,
                flecs::fs::path{G_FILE_CONTAINER},
                flecs::fs::path{G_FILE_LOCAL}))
            .Times(1);
        deployment->copy_file_from_image(G_IMAGE, G_FILE_CONTAINER, G_FILE_LOCAL);

        EXPECT_CALL(
            test_deployment,
            do_copy_file_to_instance(p, flecs::fs::path{G_FILE_LOCAL}, flecs::fs::path{G_FILE_CONTAINER}))
            .Times(1);
        deployment->copy_file_to_instance(p, G_FILE_LOCAL, G_FILE_CONTAINER);

        EXPECT_CALL(
            test_deployment,
            do_copy_file_from_instance(p, flecs::fs::path{G_FILE_CONTAINER}, flecs::fs::path{G_FILE_LOCAL}))
            .Times(1);
        deployment->copy_file_from_instance(p, G_FILE_CONTAINER, G_FILE_LOCAL);
    }
}

TEST(deployment, load_save)
{
    auto save_deployment =
        std::unique_ptr<flecs::deployments::deployment_t>{new flecs::deployments::mock_deployment_t{}};
    auto& save_uut = static_cast<flecs::deployments::mock_deployment_t&>(*save_deployment.get());

    auto instance_1 = flecs::instances::instance_t{G_INSTANCE_ID_1, app_1, G_INSTANCE_NAME_1};
    auto instance_2 = flecs::instances::instance_t{G_INSTANCE_ID_2, app_2, G_INSTANCE_NAME_2};

    save_deployment->insert_instance(instance_1);
    save_deployment->insert_instance(instance_2);
    EXPECT_CALL(save_uut, do_deployment_id()).Times(1).WillOnce(testing::Return("test"));
    save_deployment->save(".");

    auto load_deployment =
        std::unique_ptr<flecs::deployments::deployment_t>{new flecs::deployments::mock_deployment_t{}};
    auto& load_uut = static_cast<flecs::deployments::mock_deployment_t&>(*load_deployment.get());

    EXPECT_CALL(load_uut, do_deployment_id()).Times(1).WillOnce(testing::Return("test"));
    load_deployment->load(".");

    ASSERT_EQ(load_deployment->instance_ids().size(), 2);
    ASSERT_EQ(*(load_deployment->query_instance(G_INSTANCE_ID_1)), instance_1);
    ASSERT_EQ(*(load_deployment->query_instance(G_INSTANCE_ID_2)), instance_2);
}

TEST(deployment, generate_ip_success)
{
    auto deployment =
        std::unique_ptr<flecs::deployments::deployment_t>{new flecs::deployments::mock_deployment_t{}};

    {
        const auto ip = deployment->generate_instance_ip(G_CIDR_SUBNET, G_GATEWAY);
        EXPECT_EQ(ip, "172.20.0.2");
    }

    auto instance = flecs::instances::instance_t{app_1, G_INSTANCE_NAME_1};
    instance.networks().emplace_back(flecs::instances::instance_t::network_t{
        .network_name = "flecs-network",
        .mac_address = {},
        .ip_address = G_IP});

    deployment->insert_instance(instance);

    {
        const auto ip = deployment->generate_instance_ip(G_CIDR_SUBNET, G_GATEWAY);
        EXPECT_EQ(ip, "172.20.0.3");
    }
}

TEST(deployment, generate_ip_fail)
{
    auto deployment =
        std::unique_ptr<flecs::deployments::deployment_t>{new flecs::deployments::mock_deployment_t{}};

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
