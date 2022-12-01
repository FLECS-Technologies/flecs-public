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

#include "libflecs.h"

int main(int argc, char** argv)
{
    auto lib = FLECS::libflecs_t{};
    const auto host = "http://localhost";
    const auto port = 8951;

    if (lib.connect(host, port) != 0) {
        std::fprintf(stderr, "Could not connect to FLECS at %s:%d. Is the FLECS daemon running?\n", host, port);
        exit(1);
    }
    lib.run_command(argc, argv);
    std::fprintf(stdout, "%s\n", lib.json_response().c_str());
    return 0;
}
