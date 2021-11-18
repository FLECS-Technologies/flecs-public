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

#ifndef FLECS_llhttp_ext_h
#define FLECS_llhttp_ext_h

#include "external/llhttp-2.1.4/build/llhttp.h"

#include <string>

namespace FLECS {

struct llhttp_ext_t : public llhttp_t
{
    std::string _body;
    std::string _url;
};

int llhttp_ext_on_body(llhttp_t* llhttp, const char* at, size_t length);
int llhttp_ext_on_url(llhttp_t* llhttp, const char* at, size_t length);
int llhttp_ext_on_message_complete(llhttp_t* llhttp);

} // namespace FLECS


#endif //FLECS_llhttp_ext_h
