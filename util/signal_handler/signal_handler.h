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

#ifndef CCC70E82_3BE2_46D4_AA04_A1B11B6CBF5D
#define CCC70E82_3BE2_46D4_AA04_A1B11B6CBF5D

#include <atomic>

namespace FLECS {

extern std::atomic_bool g_stop;

void signal_handler(int signum);

void signal_handler_init();

} // namespace FLECS

#endif // CCC70E82_3BE2_46D4_AA04_A1B11B6CBF5D
