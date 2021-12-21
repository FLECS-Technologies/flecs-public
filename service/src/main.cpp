// Copyright 2021 FLECS Technologies GmbH
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

#include <getopt.h>
#include <unistd.h>

#include <iostream>

#include "service/service_table.h"

constexpr struct option options[] = {{"json", no_argument, nullptr, 0}, {nullptr, no_argument, nullptr, 0}};

int print_usage()
{
    printf("Usage: flecs [OPTIONS] COMMAND\n\n");
    printf("Options:\n");
    printf("    --json         Produce output in JSON format\n");
    printf("\n");
    printf("Commands:\n");
    printf("    app-manager    Manage apps and instances\n");
    printf("    help           Display help for specific COMMAND\n");
    printf("    rpc            Issue RPC for running app\n");
    printf("\n");

    return 1;
}

int main(int argc, char** argv)
{
    auto longindex = int{};
    auto opt = getopt_long(argc, argv, "", options, &longindex);
    while (opt != -1)
    {
        switch (longindex)
        {
            case 0: {
                std::cout << "Setting output to JSON format" << std::endl;
                break;
            }
        }
        opt = getopt_long(argc, argv, "", options, &longindex);
    }

    if (argc - optind < 2)
    {
        return print_usage();
    }

    const auto command{argv[optind]};

    auto it = FLECS::make_service_table.find(command);
    if (it != FLECS::make_service_table.end())
    {
        auto res = it->second()->process(argc - optind - 1, &argv[optind + 1]);
        return res != 0 ? 1 : 0;
    } else
    {
        std::cerr << "Unknown command " << command << "\n\n";
        return print_usage();
    }
}
