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

#define __STDC_FORMAT_MACROS 1

#include <cinttypes>
#include <csignal>
#include <thread>

#include "flunder/flunder_client.h"

namespace {
bool g_stop;
} // namespace

void signal_handler(int)
{
    g_stop = 1;
}

void flunder_receive_callback(FLECS::flunder_client_t* client, FLECS::flunder_data_t* msg)
{
    std::fprintf(
        stdout,
        "Received flunder message for topic %s on client %p with length %" PRIu64 "\n",
        msg->_path,
        client,
        msg->_len);
}

void flunder_receive_callback_userp(FLECS::flunder_client_t* client, FLECS::flunder_data_t* msg, const void* userp)
{
    std::fprintf(
        stdout,
        "Received flunder message for topic %s on client %p with length %" PRIu64 " and userdata %s\n",
        msg->_path,
        client,
        msg->_len,
        (const char*)userp);
}

int main()
{
    signal(SIGINT, &signal_handler);
    signal(SIGTERM, &signal_handler);

    auto flunder_client = FLECS::flunder_client_t{};

    flunder_client.connect();
    flunder_client.add_mem_storage("flunder-cpp", "/flecs/flunder/**");

    flunder_client.subscribe("/flecs/flunder/cpp", &flunder_receive_callback);
    const char* userdata = "Hello, world!";
    flunder_client.subscribe("/flecs/flunder/external", &flunder_receive_callback_userp, (const void*)userdata);

    while (!g_stop)
    {
        int i = 1234;
        flunder_client.publish("/flecs/flunder/cpp", i);
        std::this_thread::sleep_for(std::chrono::seconds(5));
    }
}
