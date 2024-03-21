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

#include <fstream>

#include "flecs/common/app/manifest/manifest.h"
#include "flecs/util/fs/fs.h"

class manifest_writer_t
{
public:
    manifest_writer_t(const char* filename, const std::string& content)
        : _filename{filename}
    {
        std::ofstream{filename} << content;
    }

    ~manifest_writer_t() { flecs::fs::remove(_filename); }

    auto& filename() const noexcept { return _filename; }

private:
    std::string _filename;
};

#define G_YAML_ITEM(I) "- " I "\n"
#define G_YAML_KEY(KEY) KEY ":\n"
#define G_YAML_KEY_VALUE(KEY, VALUE) KEY ": " VALUE "\n"
#define G_YAML_MAPPING(FROM, TO) "- " FROM ":" TO "\n"
#define G_YAML_INDENT "  "

#define G_APP "tech.flecs.test-app"
#define G_ARG_1 "--launch-arg1"
#define G_ARG_2 "--launch-arg2"
#define G_ARG_3 "launch-arg3"
#define G_ARGS                         \
    G_YAML_KEY("args")                 \
    G_YAML_INDENT G_YAML_ITEM(G_ARG_1) \
    G_YAML_INDENT G_YAML_ITEM(G_ARG_2) \
    G_YAML_INDENT G_YAML_ITEM(G_ARG_3)
#define G_AUTHOR "FLECS Technologies GmbH (info@flecs.tech)"
#define G_CATEGORY "test"
#define G_CONFFILE_LOCAL "local.conf"
#define G_CONFFILE_CONTAINER "/etc/container.conf"
#define G_CONFFILES         \
    G_YAML_KEY("conffiles") \
    G_YAML_INDENT G_YAML_MAPPING(G_CONFFILE_LOCAL, G_CONFFILE_CONTAINER)
#define G_DESCRIPTION "FLECS test app for unit tests"
#define G_DEVICE "/dev/device0"
#define G_DEVICES         \
    G_YAML_KEY("devices") \
    G_YAML_INDENT G_YAML_ITEM(G_DEVICE)
#define G_ENV_VAR1_KEY "MY_ENV_VAR"
#define G_ENV_VAR1_VALUE "ENV_VAR_VALUE"
#define G_ENV_VAR2_KEY "my.other.env"
#define G_ENV_VAR2_VALUE "MY_OTHER_VALUE"
#define G_ENV_VAR3_KEY "my.env.with.spaces"
#define G_ENV_VAR3_VALUE "Value with spaces"
#define G_ENV_VAR4_KEY "my-env-with-dashes"
#define G_ENV_VAR4_VALUE "value-with-dashes"
#define G_ENVS                                                     \
    G_YAML_KEY("env")                                              \
    G_YAML_INDENT G_YAML_MAPPING(G_ENV_VAR1_KEY, G_ENV_VAR1_VALUE) \
    G_YAML_INDENT G_YAML_MAPPING(G_ENV_VAR2_KEY, G_ENV_VAR2_VALUE) \
    G_YAML_INDENT G_YAML_MAPPING(G_ENV_VAR3_KEY, G_ENV_VAR3_VALUE) \
    G_YAML_INDENT G_YAML_MAPPING(G_ENV_VAR4_KEY, G_ENV_VAR4_VALUE)
#define G_IMAGE "flecs/test-app"
#define G_NETWORK_SETTINGS_KEY_1 "macAddress"
#define G_NETWORK_SETTINGS_VALUE_1 "clone:eth0"
#define G_NETWORK_SETTINGS        \
    G_YAML_KEY("networkSettings") \
    G_YAML_INDENT G_YAML_MAPPING(G_NETWORK_SETTINGS_KEY_1, G_NETWORK_SETTINGS_VALUE_1)
#define G_PORT_CONTAINER_1 "1234"
#define G_PORT_CONTAINER_2 "10000-10005"
#define G_PORT_LOCAL_1 "1234"
#define G_PORT_LOCAL_2 "8000-8005"
#define G_PORTS                                                      \
    G_YAML_KEY("ports")                                              \
    G_YAML_INDENT G_YAML_MAPPING(G_PORT_LOCAL_1, G_PORT_CONTAINER_1) \
    G_YAML_INDENT G_YAML_MAPPING(G_PORT_LOCAL_2, G_PORT_CONTAINER_2)
#define G_STARTUP_OPTION_1 "initNetworkAfterStart"
#define G_STARTUP_OPTIONS        \
    G_YAML_KEY("startupOptions") \
    G_YAML_INDENT G_YAML_ITEM(G_STARTUP_OPTION_1)
#define G_TITLE "FLECS test app"
#define G_VERSION "1.2.3.4-f1"
#define G_VOLUME_CONTAINER_1 "/var/"
#define G_VOLUME_CONTAINER_2 "/etc/"
#define G_VOLUME_CONTAINER_3 "/home/"
#define G_VOLUME_LOCAL_1 "var"
#define G_VOLUME_LOCAL_2 "etc"
#define G_VOLUME_LOCAL_3 "/home/app1/dir"
#define G_VOLUMES                                                        \
    G_YAML_KEY("volumes")                                                \
    G_YAML_INDENT G_YAML_MAPPING(G_VOLUME_LOCAL_1, G_VOLUME_CONTAINER_1) \
    G_YAML_INDENT G_YAML_MAPPING(G_VOLUME_LOCAL_2, G_VOLUME_CONTAINER_2) \
    G_YAML_INDENT G_YAML_MAPPING(G_VOLUME_LOCAL_3, G_VOLUME_CONTAINER_3)

auto manifest_header() //
    -> std::string
{
    return std::string{                                       //
                       G_YAML_KEY_VALUE("app", G_APP)         //
                       G_YAML_KEY_VALUE("version", G_VERSION) //
                       G_YAML_KEY_VALUE("image", G_IMAGE)};
}

auto extend_manifest(std::string& yaml) //
    -> void
{
    yaml.append(G_ARGS);
    yaml.append(G_CONFFILES);
    yaml.append(G_DEVICES);
    yaml.append(G_ENVS);
    yaml.append(G_YAML_KEY_VALUE("hostname", "flecs-unit-test"));
    yaml.append(G_YAML_KEY_VALUE("interactive", "true"));
    yaml.append(G_YAML_KEY_VALUE("multiInstance", "false"));
    yaml.append(G_NETWORK_SETTINGS);
    yaml.append(G_PORTS);
    yaml.append(G_STARTUP_OPTIONS);
    yaml.append(G_VOLUMES);
}

TEST(daemon_app, minimal_app)
{
    auto manifest = manifest_writer_t{"minimal_app.yml", manifest_header()};

    auto app = flecs::app_manifest_t::from_yaml_file(manifest.filename());

    ASSERT_TRUE(app.is_valid());
    ASSERT_EQ(app.app(), G_APP);
    ASSERT_EQ(app.version(), G_VERSION);
    ASSERT_EQ(app.image(), G_IMAGE);
    ASSERT_EQ(app.image_with_tag(), G_IMAGE ":" G_VERSION);
}

TEST(daemon_app, empty_app)
{
    auto manifest = manifest_writer_t{"empty_app.yml", std::string{}};

    auto app = flecs::app_manifest_t::from_yaml_file(manifest.filename());

    ASSERT_FALSE(app.is_valid());
}

TEST(daemon_app, complex_app)
{
    auto yaml = manifest_header();
    extend_manifest(yaml);

    auto app = flecs::app_manifest_t::from_yaml_string(yaml);

    ASSERT_TRUE(app.is_valid());
    ASSERT_EQ(app.hostname(), "flecs-unit-test");
    ASSERT_EQ(app.multi_instance(), false);
    ASSERT_EQ(app.interactive(), true);
    ASSERT_EQ(
        (*app.env().cbegin()),
        (flecs::mapped_env_var_t{flecs::env_var_t{G_ENV_VAR1_KEY}, G_ENV_VAR1_VALUE}));
    ASSERT_EQ(
        (*(++app.env().cbegin())),
        (flecs::mapped_env_var_t{flecs::env_var_t{G_ENV_VAR4_KEY}, G_ENV_VAR4_VALUE}));
    ASSERT_EQ(
        (std::find_if(
             app.volumes().cbegin(),
             app.volumes().cend(),
             [](const flecs::volume_t& v) { return v.host() == G_VOLUME_LOCAL_1; })
             ->container()),
        G_VOLUME_CONTAINER_1);
    ASSERT_EQ(
        (std::find_if(
             app.volumes().cbegin(),
             app.volumes().cend(),
             [](const flecs::volume_t& v) { return v.host() == G_VOLUME_LOCAL_2; })
             ->container()),
        G_VOLUME_CONTAINER_2);
    ASSERT_EQ(
        (std::find_if(
             app.volumes().cbegin(),
             app.volumes().cend(),
             [](const flecs::volume_t& v) { return v.host() == G_VOLUME_LOCAL_3; })
             ->container()),
        G_VOLUME_CONTAINER_3);
    ASSERT_EQ((app.args()[0]), G_ARG_1);
    ASSERT_EQ((app.args()[1]), G_ARG_2);
    ASSERT_EQ((app.args()[2]), G_ARG_3);
    ASSERT_EQ((app.networks()[0].name()), "flecs");
    ASSERT_EQ((app.networks()[0].type()), flecs::network_type_e::Bridge);
    ASSERT_EQ((app.ports()[0].host_port_range()), flecs::port_range_t{G_PORT_LOCAL_1});
    ASSERT_EQ((app.ports()[0].container_port_range()), flecs::port_range_t{G_PORT_CONTAINER_1});
    ASSERT_EQ((app.ports()[1].host_port_range()), flecs::port_range_t{G_PORT_LOCAL_2});
    ASSERT_EQ((app.ports()[1].container_port_range()), flecs::port_range_t{G_PORT_CONTAINER_2});
    ASSERT_EQ((app.conffiles()[0].local()), G_CONFFILE_LOCAL);
    ASSERT_EQ((app.conffiles()[0].container()), G_CONFFILE_CONTAINER);
    ASSERT_EQ((*app.devices().begin()), G_DEVICE);
    ASSERT_EQ((app.startup_options()[0]), flecs::startup_option_t::INIT_NETWORK_AFTER_START);
}

TEST(daemon_app, invalid_file)
{
    const auto app_manifest = flecs::app_manifest_t::from_yaml_file("/no/such/manifest.yml");

    ASSERT_FALSE(app_manifest.is_valid());
}

TEST(daemon_app, to_json)
{
    auto yaml = manifest_header();
    extend_manifest(yaml);

    const auto app_manifest = flecs::app_manifest_t::from_yaml_string(yaml);

    const auto json = flecs::json_t(app_manifest);
    const auto json_expected =
        R"-({"app":"tech.flecs.test-app",)-"
        R"-("version":"1.2.3.4-f1",)-"
        R"-("image":"flecs/test-app",)-"
        R"-("multiInstance":false,)-"
        R"-("editor":"",)-"
        R"-("args":["--launch-arg1","--launch-arg2","launch-arg3"],)-"
        R"-("capabilities":[],)-"
        R"-("conffiles":["local.conf:/etc/container.conf:rw"],)-"
        R"-("devices":["/dev/device0"],)-"
        R"-("env":["MY_ENV_VAR=ENV_VAR_VALUE","my-env-with-dashes=value-with-dashes","my.env.with.spaces=Value with spaces","my.other.env=MY_OTHER_VALUE"],)-"
        R"-("hostname":"flecs-unit-test",)-"
        R"-("interactive":true,)-"
        R"-("networks":[{"mac_address":"","name":"flecs","parent":"","type":"bridge"}],)-"
        R"-("ports":["1234:1234","8000-8005:10000-10005"],)-"
        R"-("startupOptions":[1],)-"
        R"-("volumes":["var:/var/","etc:/etc/","/home/app1/dir:/home/"]})-";

    ASSERT_EQ(json.dump(), json_expected);
}
