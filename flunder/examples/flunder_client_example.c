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

#include "flunder/flunder_client.h"

#include <signal.h>
#include <stdio.h>
#include <unistd.h>

static int g_stop;

void signal_handler(int signum)
{
    (void)signum;
    g_stop = 1;
}

void flunder_subscribe_callback(void* client, struct flunder_data_t* msg)
{
    fprintf(stdout, "Received flunder message for topic %s on client %p\n", msg->_topic, client);
}

int main(void)
{
    signal(SIGINT, &signal_handler);
    signal(SIGTERM, &signal_handler);

    void* flunder_client = flunder_client_new();

    flunder_connect(flunder_client, FLECS_FLUNDER_HOST, FLECS_FLUNDER_PORT);
    flunder_add_mem_storage(flunder_client, "flunder-c", "/flecs/flunder/**");
    flunder_subscribe(flunder_client, "/flecs/flunder/c", &flunder_subscribe_callback);
    flunder_subscribe(flunder_client, "/flecs/flunder/external", &flunder_subscribe_callback);

    while (!g_stop)
    {
        int i = 1234;
        flunder_publish_int(flunder_client, "/flecs/flunder/c", i);
        sleep(5);
    }

    flunder_remove_mem_storage(flunder_client, "flunder-c");
    flunder_client_destroy(flunder_client);
}
