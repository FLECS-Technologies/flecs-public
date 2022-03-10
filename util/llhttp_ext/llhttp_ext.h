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

#ifndef D1251A7E_3083_4C6E_8B7E_C02B589C00AA
#define D1251A7E_3083_4C6E_8B7E_C02B589C00AA

#include <llhttp.h>

#include <string>

#include "util/http/status_codes.h"

namespace FLECS {

struct llhttp_ext_t : public llhttp_t
{
    std::string _body;
    std::string _url;
};

void llhttp_ext_settings_init(llhttp_settings_t* settings);
int llhttp_ext_on_body(llhttp_t* llhttp, const char* at, size_t length);
int llhttp_ext_on_url(llhttp_t* llhttp, const char* at, size_t length);
int llhttp_ext_on_message_complete(llhttp_t* llhttp);

} // namespace FLECS

#endif // D1251A7E_3083_4C6E_8B7E_C02B589C00AA
