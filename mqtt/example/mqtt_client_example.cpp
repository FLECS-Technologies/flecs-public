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

#include <csignal>
#include <thread>

#include "mqtt/mqtt_client.h"

namespace {
bool g_stop;
} // namespace

void signal_handler(int)
{
    g_stop = 1;
}

void mqtt_receive_callback(FLECS::mqtt_client_t* client, FLECS::mqtt_message_t* msg, void*)
{
    std::fprintf(stdout, "Received MQTT message for topic %s on client %p\n", msg->topic, client);
}

int main()
{
    signal(SIGINT, &signal_handler);
    signal(SIGTERM, &signal_handler);

    auto flecs_mqtt = FLECS::mqtt_client_t{};

    flecs_mqtt.receive_callback_set(&mqtt_receive_callback, nullptr);
    flecs_mqtt.connect();

    flecs_mqtt.subscribe("/flecs/test/cpp", 0);
    flecs_mqtt.subscribe("/flecs/test/external", 0);

    while (!g_stop)
    {
        int i = 1234;
        flecs_mqtt.publish("/flecs/test/cpp", sizeof(i), &i, 0, false);
        std::this_thread::sleep_for(std::chrono::seconds(5));
    }
}
