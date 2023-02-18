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

#include <signal.h>
#include <stdio.h>
#include <unistd.h>

#include "mqtt/mqtt_client.h"

static int g_stop;

void signal_handler(int signum)
{
    (void)signum;
    g_stop = 1;
}

void mqtt_receive_callback(void* client, struct flecs_mqtt_message_t* msg, void* userp)
{
    (void)userp;
    fprintf(stdout, "Received MQTT message for topic %s on client %p\n", msg->topic, client);
}

int main(void)
{
    signal(SIGINT, &signal_handler);
    signal(SIGTERM, &signal_handler);

    void* mqtt_client = flecs_mqtt_client_new();

    flecs_mqtt_receive_callback_set(mqtt_client, &mqtt_receive_callback, NULL);
    flecs_mqtt_connect(mqtt_client, FLECS_MQTT_HOST, FLECS_MQTT_PORT, FLECS_MQTT_KEEPALIVE);

    flecs_mqtt_subscribe(mqtt_client, "/flecs/test/c", 0);
    flecs_mqtt_subscribe(mqtt_client, "/flecs/test/external", 0);

    while (!g_stop) {
        int i = 1234;
        flecs_mqtt_publish(mqtt_client, "/flecs/test/c", sizeof(i), &i, 0, false);
        sleep(5);
    }

    flecs_mqtt_client_destroy(mqtt_client);
}
