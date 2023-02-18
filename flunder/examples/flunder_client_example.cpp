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

#define __STDC_FORMAT_MACROS 1

#include <chrono>
#include <cinttypes>
#include <csignal>
#include <cstring>
#include <string>
#include <string_view>
#include <thread>

#include "flunder/flunder_client.h"

namespace {
bool g_stop;
} // namespace

void signal_handler(int)
{
    g_stop = 1;
}

void flunder_receive_callback(FLECS::flunder_client_t* client, const FLECS::flunder_variable_t* var)
{
    const auto now = std::chrono::high_resolution_clock::now().time_since_epoch().count();
    std::fprintf(
        stdout,
        "Received flunder message for topic %s on client %p with length %zu @%" PRIi64 "\n",
        var->topic().data(),
        client,
        var->len(),
        now);

    if (var->topic() == "/flecs/flunder/cpp/int") {
        const auto i = std::atoll(var->value().data());
        std::fprintf(stdout, "\tValue: %lld\n", i);
    } else if (var->topic() == "/flecs/flunder/cpp/double") {
        const auto d = std::atof(var->value().data());
        std::fprintf(stdout, "\tValue: %lf\n", d);
    } else if (var->topic() == "/flecs/flunder/cpp/string") {
        std::fprintf(stdout, "\tValue: %s\n", var->value().data());
    } else if (var->topic() == "/flecs/flunder/cpp/timestamp") {
        const auto t1 = std::stoll(var->value().data());
        const auto diff = now - t1;
        std::fprintf(stdout, "\tMessage sent @%lld (%lld ns ago)\n", t1, diff);
    }
}

void flunder_receive_callback_userp(
    FLECS::flunder_client_t* client, const FLECS::flunder_variable_t* var, const void* userp)
{
    const auto timestamp = std::chrono::high_resolution_clock::now().time_since_epoch().count();
    std::fprintf(
        stdout,
        "Received flunder message for topic %s on client %p with length %zu and userdata %s "
        "@%" PRIi64 "\n",
        var->topic().data(),
        client,
        var->len(),
        (const char*)userp,
        timestamp);
}

int main()
{
    signal(SIGINT, &signal_handler);
    signal(SIGTERM, &signal_handler);

    auto flunder_client = FLECS::flunder_client_t{};

    flunder_client.connect();
    flunder_client.add_mem_storage("flunder-cpp", "/flecs/flunder/**");

    flunder_client.subscribe("/flecs/flunder/cpp/**", &flunder_receive_callback);
    const char* userdata = "Hello, world!";
    flunder_client.subscribe(
        "/flecs/flunder/external",
        &flunder_receive_callback_userp,
        (const void*)userdata);

    while (!g_stop) {
        const auto i = 1234;
        flunder_client.publish("/flecs/flunder/cpp/int", i);

        const auto d = 3.14159;
        flunder_client.publish("/flecs/flunder/cpp/double", d);

        const auto str = "Hello, world!";
        flunder_client.publish("/flecs/flunder/cpp/string", str);

        const auto t = std::chrono::high_resolution_clock::now().time_since_epoch().count();
        flunder_client.publish("/flecs/flunder/cpp/timestamp", t);

        std::this_thread::sleep_for(std::chrono::seconds(5));
    }
}
