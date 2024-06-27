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

const auto raw_json_manifest =
    R"-({"app":"tech.flecs.test-app",)-"
    R"-("_schemaVersion":"3.0.0",)-"
    R"-("version":"1.2.3.4-f1",)-"
    R"-("image":"flecs/test-app",)-"
    R"-("multiInstance":false,)-"
    R"-("editors":[{"name":"Editor","port":1234,"supportsReverseProxy":true}],)-"
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
    R"-("volumes":["var:/var/","etc:/etc/","/home/app1/dir:/home/"],)-"
    R"-("labels":["empty=","some.json={\"varname\": 123}","with-whitespace=some\tvalue with\nwhitespace"]})-";


TEST(daemon_app, json)
{
    const auto app_manifest = flecs::app_manifest_t::from_json_string(raw_json_manifest);

    const auto json = flecs::json_t(app_manifest);
    ASSERT_EQ(json.dump(), raw_json_manifest);
}
