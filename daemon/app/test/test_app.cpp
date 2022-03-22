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

#include <filesystem>
#include <fstream>

#include "daemon/app/app.h"
#include "gtest/gtest.h"

class manifest_writer_t
{
public:
    manifest_writer_t(const char* filename, const std::string& content)
        : _filename{filename}
    {
        std::ofstream{filename} << content;
    }

    ~manifest_writer_t() { std::filesystem::remove(_filename); }

    auto& filename() const noexcept { return _filename; }

private:
    std::string _filename;
};

auto g_app = "tech.flecs.test-app";
auto g_title = "FLECS test app";
auto g_version = "1.2.3.4-f1";
auto g_description = "FLECS test app for unit tests";
auto g_author = "FLECS Technologies GmbH (info@flecs.tech)";
auto g_category = "test";
auto g_image = "flecs/test-app";

std::string manifest_header()
{
    return std::string{
        "app: tech.flecs.test-app\n"
        "title: FLECS test app\n"
        "version: 1.2.3.4-f1\n"
        "author: FLECS Technologies GmbH (info@flecs.tech)\n"
        "image: flecs/test-app\n"};
}

TEST(daemon_app, minimal_app)
{
    auto manifest = manifest_writer_t{"minimal_app.yml", manifest_header()};

    auto app = FLECS::app_t{manifest.filename()};

    ASSERT_TRUE(app.yaml_loaded());
    ASSERT_EQ(app.name(), g_app);
    ASSERT_EQ(app.title(), g_title);
    ASSERT_EQ(app.version(), g_version);
    ASSERT_EQ(app.author(), g_author);
    ASSERT_EQ(app.image(), g_image);
}

TEST(daemon_app, empty_app)
{
    auto manifest = manifest_writer_t{"empty_app.yml", std::string{}};

    auto app = FLECS::app_t{manifest.filename()};

    ASSERT_FALSE(app.yaml_loaded());
}

TEST(daemon_app, complex_app)
{
    auto yaml = manifest_header();
    yaml.append("description: FLECS Test application\n");
    yaml.append("category: test\n");
    yaml.append(
        "args:\n"
        "  - --launch-arg1\n"
        "  - --launch-arg2\n"
        "  - launch-arg3\n");
    yaml.append(
        "env:\n"
        "  - MY_ENV_VAR:ENV_VALUE\n");
    yaml.append(
        "ports:\n"
        "  - 1234:1234\n"
        "  - 8000-8005:10000-10005\n");
    yaml.append(
        "volumes:\n"
        "  - var:/var/\n"
        "  - etc:/etc/\n"
        "  - /home/app1/dir:/home/\n");
    yaml.append("multiInstance: false\n");
    yaml.append("hostname: flecs-unit-test\n");
    yaml.append("interactive: true\n");

    auto manifest = manifest_writer_t{"complex_app.yml", yaml};

    auto app = FLECS::app_t{manifest.filename()};

    ASSERT_TRUE(app.yaml_loaded());
    ASSERT_EQ(app.description(), "FLECS Test application");
    ASSERT_EQ(app.category(), "test");
    ASSERT_EQ(app.hostname(), "flecs-unit-test");
    ASSERT_EQ(app.multi_instance(), false);
    ASSERT_EQ(app.interactive(), true);
    ASSERT_EQ((app.env()[0]), (FLECS::mapped_env_var_t{FLECS::env_var_t{"MY_ENV_VAR"}, "ENV_VALUE"}));
    ASSERT_EQ((app.volumes().at("var")), "/var/");
    ASSERT_EQ((app.volumes().at("etc")), "/etc/");
    ASSERT_EQ((app.bind_mounts().at("/home/app1/dir")), "/home/");
    ASSERT_EQ((app.args()[0]), "--launch-arg1");
    ASSERT_EQ((app.args()[1]), "--launch-arg2");
    ASSERT_EQ((app.args()[2]), "launch-arg3");
    ASSERT_EQ((app.ports()[0]), FLECS::mapped_port_range_t{"1234:1234"});
    ASSERT_EQ((app.ports()[1]), FLECS::mapped_port_range_t{"8000-8005:10000-10005"});
}

TEST(daemon_app, invalid_apps)
{
    {
        auto yaml = manifest_header();
        yaml.append("hostname: flecs-unit-test\n");
        yaml.append("multiInstance: true\n");

        auto manifest = manifest_writer_t{"invalid_app.yml", yaml};

        auto app = FLECS::app_t{manifest.filename()};
        ASSERT_FALSE(app.yaml_loaded());
        ASSERT_EQ(app.name(), "");
    }
}
