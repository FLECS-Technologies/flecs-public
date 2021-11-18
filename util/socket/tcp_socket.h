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

#ifndef FLECS_util_tcp_socket_h
#define FLECS_util_tcp_socket_h

#include "socket_base.h"

namespace FLECS {

class tcp_socket_t : public socket_t
{
public:
    tcp_socket_t()
        : socket_t { domain_t::INET, type_t::STREAM, 0 } {}
    tcp_socket_t(int fd)
        : socket_t { fd } {}
};

} // namespace FLECS

#endif //FLECS_util_tcp_socket_h
