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

#include <sstream>

#include "daemon/common/db/app_db.h"
#include "gtest/gtest.h"
#include "util/fs/fs.h"

namespace {

auto make_app_entry(std::string app = "tech.flecs.test", std::string version = "1.0.0.0")
{
    auto manifest_string = std::stringstream{};
    manifest_string << "app: \"" << app << "\"\n";
    manifest_string << "title: \"FLECS Test App\"\n";
    manifest_string << "version: \"" << version << "\"\n";
    manifest_string << "author: \"FLECS Technologies GmbH\"\n";
    manifest_string << "category: \"test\"\n";
    manifest_string << "image: \"flecs/" << app << "\"\n";

    return FLECS::app_t{manifest_string.str(), FLECS::NOT_INSTALLED, FLECS::INSTALLED};
}

auto make_instance_entry(std::string id = "789abcde", const FLECS::app_t& app = make_app_entry())
{
    return FLECS::instances_table_entry_t{
        FLECS::instances_table_primary_t{id},
        FLECS::instances_table_data_t{
            app.app(),
            app.version(),
            "Test instance",
            FLECS::NOT_CREATED,
            FLECS::CREATED,
            {"network"},
            {"127.0.0.1"},
            0}};
}

} // namespace

void assert_db_has_app(const FLECS::app_db_t& app_db, const FLECS::app_t& app)
{
    const auto primary = FLECS::apps_table_primary_t{app.app(), app.version()};
    const auto data = FLECS::apps_table_data_t{
        app.status(),
        app.desired(),
        app.category(),
        app.installed_size(),
        app.license_key(),
        app.download_token()};

    ASSERT_TRUE(app_db.has_app(primary));

    const auto app_opt = app_db.query_app(primary);
    ASSERT_TRUE(app_opt.has_value());

    decltype(auto) app_val = app_opt.value();

    ASSERT_EQ(app_val.app, primary.app);
    ASSERT_EQ(app_val.version, primary.version);
    ASSERT_EQ(app_val.status, data.status);
    ASSERT_EQ(app_val.desired, data.desired);
    ASSERT_EQ(app_val.category, data.category);
    ASSERT_EQ(app_val.installed_size, data.installed_size);
    ASSERT_EQ(app_val.license_key, data.license_key);
    ASSERT_EQ(app_val.download_token, data.download_token);
}

void assert_db_has_instance(const FLECS::app_db_t& app_db, const FLECS::instances_table_entry_t& instance)
{
    decltype(auto) primary = static_cast<const FLECS::instances_table_primary_t&>(instance);
    decltype(auto) data = static_cast<const FLECS::instances_table_data_t&>(instance);

    ASSERT_TRUE(app_db.has_instance(primary));

    const auto instance_opt = app_db.query_instance(primary);
    ASSERT_TRUE(instance_opt.has_value());

    decltype(auto) instance_val = instance_opt.value();

    ASSERT_EQ(instance_val.id, primary.id);
    ASSERT_EQ(instance_val.app, data.app);
    ASSERT_EQ(instance_val.version, data.version);
    ASSERT_EQ(instance_val.status, data.status);
    ASSERT_EQ(instance_val.desired, data.desired);
    ASSERT_EQ(instance_val.description, data.description);
    ASSERT_EQ(instance_val.networks, data.networks);
    ASSERT_EQ(instance_val.ips, data.ips);
    ASSERT_EQ(instance_val.flags, data.flags);
}

const auto app_db_path = fs::current_path().string() + "/apps.db";

TEST(service_app_manager_app_db, CreateDatabase)
{
    fs::remove(app_db_path);
    auto app_db = FLECS::app_db_t{app_db_path};
    ASSERT_TRUE(fs::exists(app_db_path));
}

TEST(service_app_manager_app_db, InsertAndDeleteApp)
{
    auto app_db = FLECS::app_db_t{app_db_path};
    const auto app = make_app_entry();
    decltype(auto) primary = FLECS::apps_table_primary_t{app.app(), app.version()};

    ASSERT_FALSE(app_db.has_app(primary));
    ASSERT_FALSE(app_db.query_app(primary).has_value());

    app_db.insert_app(app);
    const auto all_apps = app_db.all_apps();

    ASSERT_EQ(all_apps.size(), 1);

    assert_db_has_app(app_db, app);

    app_db.delete_app(primary);

    ASSERT_FALSE(app_db.has_app(primary));
    ASSERT_FALSE(app_db.query_app(primary).has_value());
}

TEST(service_app_manager_app_db, InsertAndDeleteInstance)
{
    auto app_db = FLECS::app_db_t{app_db_path};
    const auto instance = make_instance_entry();
    decltype(auto) primary = static_cast<const FLECS::instances_table_primary_t&>(instance);
    decltype(auto) data = static_cast<const FLECS::instances_table_data_t&>(instance);

    ASSERT_FALSE(app_db.has_instance(primary));
    ASSERT_FALSE(app_db.query_instance(primary).has_value());

    app_db.insert_instance({primary, data});

    const auto all_instances = app_db.all_instances();

    ASSERT_EQ(all_instances.size(), 1);

    assert_db_has_instance(app_db, instance);

    app_db.delete_instance(primary);

    ASSERT_FALSE(app_db.has_instance(primary));
    ASSERT_FALSE(app_db.query_instance(primary).has_value());
}

TEST(service_app_manager_app_db, PersistDatabase)
{
    const auto app = make_app_entry();
    const auto instance = make_instance_entry();

    {
        auto app_db = FLECS::app_db_t{app_db_path};
        app_db.insert_app(app);
        app_db.insert_instance(instance);
    }
    {
        auto app_db = FLECS::app_db_t{app_db_path};
        assert_db_has_app(app_db, app);
        assert_db_has_instance(app_db, instance);
    }
}

TEST(service_app_manager_app_db, UpdateAppAndInstance)
{
    auto app_db = FLECS::app_db_t{app_db_path};

    auto app = make_app_entry();
    app_db.insert_app(app);
    app.installed_size(8888);
    app_db.insert_app(app);
    assert_db_has_app(app_db, app);

    auto instance = make_instance_entry();
    app_db.insert_instance(instance);
    instance.description = "changed";
    app_db.insert_instance(instance);
    assert_db_has_instance(app_db, instance);
}

TEST(service_app_manager_app_db, GetAllInstances)
{
    fs::remove(app_db_path);
    auto app_db = FLECS::app_db_t{app_db_path};

    const auto apps = {
        make_app_entry("tech.flecs.test", "1.0.0.0"),
        make_app_entry("tech.flecs.test", "2.0.0.0"),
        make_app_entry("tech.flecs.another_test", "1.0.0.0"),
    };
    for (decltype(auto) app : apps)
    {
        app_db.insert_app(app);
    }

    auto instances = {
        make_instance_entry("01234567", std::data(apps)[0]),
        make_instance_entry("12345678", std::data(apps)[0]),
        make_instance_entry("23456789", std::data(apps)[0]),
        make_instance_entry("abcdef01", std::data(apps)[1]),
        make_instance_entry("98765432", std::data(apps)[1]),
        make_instance_entry("56789876", std::data(apps)[2]),
        make_instance_entry("77777777", std::data(apps)[2]),
    };

    for (decltype(auto) instance : instances)
    {
        app_db.insert_instance(instance);
    }

    auto test_instances1 = app_db.instances(std::data(apps)[0].app(), std::data(apps)[0].version());
    ASSERT_EQ(test_instances1.size(), 3);

    auto test_instances2 = app_db.instances(std::data(apps)[0].app());
    ASSERT_EQ(test_instances2.size(), 5);

    auto test_instances3 = app_db.instances(std::data(apps)[2].app(), std::data(apps)[2].version());
    ASSERT_EQ(test_instances3.size(), 2);

    auto test_instances4 = app_db.instances(std::data(apps)[2].app());
    ASSERT_EQ(test_instances4.size(), 2);
}
