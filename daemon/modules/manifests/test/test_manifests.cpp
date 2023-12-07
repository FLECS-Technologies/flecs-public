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

#include <filesystem>
#include <fstream>

#include "daemon/common/app/app_key.h"
#include "daemon/common/app/manifest/manifest.h"
#include "daemon/modules/manifests/manifests.h"

namespace fs = std::filesystem;

class test_module_manifests_t : public flecs::module::manifests_t
{
public:
    test_module_manifests_t() = default;
};

constexpr auto valid_manifest_1 = //
    "app: tech.flecs.test-app-1\n"
    "version: 1.2.3.4-f1\n"
    "author: FLECS Technologies GmbH\n"
    "title: FLECS Test App\n"
    "image: flecs/tech.flecs.test-app-1\n";
const auto valid_key_1 = flecs::app_key_t{"tech.flecs.test-app-1", "1.2.3.4-f1"};

constexpr auto valid_manifest_2 = //
    "app: tech.flecs.test-app-2\n"
    "version: 2.3.4.5-f2\n"
    "author: FLECS Technologies GmbH\n"
    "title: FLECS Test App\n"
    "image: flecs/tech.flecs.test-app-1\n";
const auto valid_key_2 = flecs::app_key_t{"tech.flecs.test-app-2", "2.3.4.5-f2"};

constexpr auto invalid_manifest_1 = //
    "app: tech.flecs.invalid-app-1\n"
    "version: 1.2.3.4-f1\n"
    "author: FLECS Technologies GmbH\n"
    "title: FLECS Test App\n"
    "image: flecs/tech.flecs.invalid-app-1\n";
const auto invalid_key_1 = flecs::app_key_t{"tech.flecs.invalid-app-1", "1.2.3.4-f1"};

constexpr auto invalid_manifest_2 = //
    "app: tech.flecs.invalid-app-2\n"
    "version: 1.2.3.4-f1\n"
    "author: FLECS Technologies GmbH\n"
    "title: FLECS Test App\n"
    "image: flecs/tech.flecs.invalid-app-2\n";
const auto invalid_key_2 = flecs::app_key_t{"tech.flecs.invalid-app-2", "1.2.3.4-f1"};

constexpr auto invalid_manifest_3 = //
    R"(%@^!@#$$%^)";

const auto json_manifest_1 = //
    R"({)"
    R"("app":"tech.flecs.test-app-1",)"
    R"("version":"1.2.3.4-f1",)"
    R"("author":"FLECS Technologies GmbH",)"
    R"("title":"FLECS Test App",)"
    R"("image":"flecs/tech.flecs.test-app-1")"
    R"(})";

const auto json_manifest_2 = //
    R"({)"
    R"("app":"tech.flecs.test-app-2",)"
    R"("version":"2.3.4.5-f2",)"
    R"("author":"FLECS Technologies GmbH",)"
    R"("title":"FLECS Test App",)"
    R"("image":"flecs/tech.flecs.test-app-2")"
    R"(})";

auto create_manifests(const fs::path& base_path) //
    -> void
{
    fs::create_directories(base_path / "tech.flecs.test-app-1/1.2.3.4-f1");
    fs::create_directories(base_path / "tech.flecs.test-app-2/2.3.4.5-f2");
    fs::create_directories(base_path / "tech.flecs.invalid-app-1/2.3.4.5-f2");
    fs::create_directories(base_path / "tech.flecs.invalid-app-2");
    fs::create_directories(base_path / "_");

    {
        auto file = std::ofstream{base_path / "tech.flecs.test-app-1/1.2.3.4-f1/manifest.yml"};
        file << valid_manifest_1;
    }
    {
        auto file = std::ofstream{base_path / "tech.flecs.test-app-1/1.2.3.4-f1/manifest.json"};
        file << json_manifest_1;
    }
    {
        auto file = std::ofstream{base_path / "tech.flecs.test-app-2/2.3.4.5-f2/manifest.yml"};
        file << valid_manifest_2;
    }
    {
        auto file = std::ofstream{base_path / "tech.flecs.invalid-app-1/2.3.4.5-f2/manifest.yml"};
        file << invalid_manifest_1;
    }
    {
        auto file = std::ofstream{base_path / "tech.flecs.invalid-app-2/manifest.yml"};
        file << invalid_manifest_2;
    }
    {
        auto file = std::ofstream{base_path / "_/!.yml"};
        file << invalid_manifest_3;
    }
}

auto delete_manifests(const fs::path& base_path) //
    -> void
{
    fs::remove_all(base_path);
}

TEST(manifests, load_success)
{
    create_manifests("./manifests");

    auto uut = test_module_manifests_t{};
    uut.init();
    uut.base_path("./manifests");
    ASSERT_EQ(uut.base_path(), fs::canonical("./manifests"));

    auto& uut_c = static_cast<const test_module_manifests_t&>(uut);

    ASSERT_FALSE(uut.contains(valid_key_1));
    ASSERT_FALSE(uut_c.contains(valid_key_2));
    ASSERT_FALSE(uut.contains(invalid_key_1));
    ASSERT_FALSE(uut_c.contains(invalid_key_2));

    ASSERT_TRUE(uut.query(valid_key_1));
    ASSERT_TRUE(uut_c.query(valid_key_2));
    ASSERT_FALSE(uut.query(invalid_key_1));
    ASSERT_FALSE(uut_c.query(invalid_key_2));

    ASSERT_TRUE(uut.contains(valid_key_1));
    ASSERT_TRUE(uut_c.contains(valid_key_2));
    ASSERT_FALSE(uut.contains(invalid_key_1));
    ASSERT_FALSE(uut_c.contains(invalid_key_2));

    ASSERT_EQ(uut.query(valid_key_1)->author(), "FLECS Technologies GmbH");
    ASSERT_EQ(uut_c.query(valid_key_2)->author(), "FLECS Technologies GmbH");

    ASSERT_EQ(uut.path(flecs::app_key_t{}), "");
    ASSERT_EQ(
        uut.path(valid_key_1),
        fs::canonical("./manifests/tech.flecs.test-app-1/1.2.3.4-f1/manifest.json"));

    uut.deinit();

    delete_manifests("./manifests");
}

TEST(manifests, load_fail)
{
    create_manifests("./import");

    auto uut = test_module_manifests_t{};
    uut.init();
    uut.base_path("/sys/class/manifests");
    ASSERT_EQ(uut.base_path(), "");

    const auto [manifest_1, res_1] =
        uut.add_from_file("./import/tech.flecs.test-app-1/1.2.3.4-f1/manifest.yml");
    ASSERT_FALSE(res_1);
    ASSERT_FALSE(uut.contains(valid_key_1));
    ASSERT_FALSE(uut.query(valid_key_1));

    uut.remove(valid_key_1);
    uut.erase(valid_key_1);

    delete_manifests("./import");
}

TEST(manifests, add_from_file)
{
    create_manifests("./import");

    auto uut = test_module_manifests_t{};
    uut.init();
    uut.base_path("./manifests");

    ASSERT_FALSE(fs::exists("./manifests/tech.flecs.test-app-1/1.2.3.4-f1/manifest.json"));

    const auto [manifest_1, res_1] =
        uut.add_from_file("./import/tech.flecs.test-app-1/1.2.3.4-f1/manifest.yml");
    ASSERT_TRUE(res_1);
    ASSERT_TRUE(uut.contains(valid_key_1));
    ASSERT_TRUE(uut.query(valid_key_1));
    ASSERT_EQ(uut.query(valid_key_1)->author(), "FLECS Technologies GmbH");

    ASSERT_TRUE(fs::exists("./manifests/tech.flecs.test-app-1/1.2.3.4-f1/manifest.json"));

    const auto [manifest_2, res_2] =
        uut.add_from_file("./import/tech.flecs.test-app-1/1.2.3.4-f1/manifest.yml");
    ASSERT_FALSE(res_2);
    ASSERT_TRUE(uut.contains(valid_key_1));
    ASSERT_TRUE(uut.query(valid_key_1));
    ASSERT_EQ(uut.query(valid_key_1)->author(), "FLECS Technologies GmbH");

    uut.deinit();

    delete_manifests("./import");
    delete_manifests("./manifests");
}

TEST(manifests, add_from_yaml_string)
{
    auto uut = test_module_manifests_t{};
    uut.init();
    uut.base_path("./manifests");

    const auto [manifest_1, res_1] = uut.add_from_string(valid_manifest_1);
    ASSERT_TRUE(res_1);
    ASSERT_TRUE(uut.contains(valid_key_1));
    ASSERT_TRUE(uut.query(valid_key_1));
    ASSERT_EQ(uut.query(valid_key_1)->author(), "FLECS Technologies GmbH");

    const auto [manifest_2, res_2] = uut.add_from_string(valid_manifest_1);
    ASSERT_FALSE(res_2);
    ASSERT_TRUE(uut.contains(valid_key_1));
    ASSERT_TRUE(uut.query(valid_key_1));
    ASSERT_EQ(uut.query(valid_key_1)->author(), "FLECS Technologies GmbH");

    uut.deinit();

    delete_manifests("./manifests");
}

TEST(manifests, add_from_json_string)
{
    auto uut = test_module_manifests_t{};
    uut.init();
    uut.base_path("./manifests");

    const auto [manifest_1, res_1] = uut.add_from_string(json_manifest_2);
    ASSERT_TRUE(res_1);
    ASSERT_TRUE(uut.contains(valid_key_2));
    ASSERT_TRUE(uut.query(valid_key_2));
    ASSERT_EQ(uut.query(valid_key_2)->author(), "FLECS Technologies GmbH");

    const auto [manifest_2, res_2] = uut.add_from_string(json_manifest_2);
    ASSERT_FALSE(res_2);
    ASSERT_TRUE(uut.contains(valid_key_2));
    ASSERT_TRUE(uut.query(valid_key_2));
    ASSERT_EQ(uut.query(valid_key_2)->author(), manifest_2->author());
    ASSERT_EQ(uut.query(valid_key_2)->author(), "FLECS Technologies GmbH");

    uut.deinit();

    delete_manifests("./manifests");
}

TEST(manifests, add_from_url)
{
    create_manifests("./import");

    auto uut = test_module_manifests_t{};
    uut.init();
    uut.base_path("./manifests");

    auto key = flecs::app_key_t{"tech.flecs.mqtt-bridge", "1.6.1-porpoise"};
    const auto [manifest_1, res_1] = uut.add_from_marketplace(key);
    ASSERT_TRUE(res_1);
    ASSERT_TRUE(uut.contains(key));
    ASSERT_TRUE(uut.query(key));
    ASSERT_EQ(uut.query(key)->title(), "FLECS MQTT bridge");

    /* not a manifest, but exceeds size limit */
    const auto [manifest_2, res_2] =
        uut.add_from_url("https://marketplace.flecs.tech/dl/deb/flecs_1.6.1-porpoise_amd64.deb");
    ASSERT_FALSE(res_2);

    delete_manifests("./import");
    delete_manifests("./manifests");
}

TEST(manifests, erase_remove)
{
    create_manifests("./manifests");

    auto uut = test_module_manifests_t{};
    uut.init();
    uut.base_path("./manifests");

    ASSERT_TRUE(uut.query(valid_key_1));
    ASSERT_TRUE(uut.query(valid_key_2));

    uut.remove(valid_key_1);

    ASSERT_FALSE(uut.contains(valid_key_1));
    ASSERT_TRUE(uut.contains(valid_key_2));

    ASSERT_TRUE(uut.query(valid_key_1));
    ASSERT_TRUE(uut.query(valid_key_2));

    uut.erase(valid_key_1);

    ASSERT_FALSE(uut.query(valid_key_1));
    ASSERT_TRUE(uut.query(valid_key_2));

    uut.erase(valid_key_1);
    uut.erase(invalid_key_1);

    uut.deinit();

    delete_manifests("./manifests");
}
