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

#ifndef CBE91B30_CC56_4E6A_969C_9A84233C1DA4
#define CBE91B30_CC56_4E6A_969C_9A84233C1DA4

#include "module_base/module.h"

namespace FLECS {

class module_marketplace_t : public module_t
{
public:
protected:
    friend class module_factory_t;

    module_marketplace_t();

    auto& user() const noexcept { return _user; }
    auto& token() const noexcept { return _token; }

private:
    http_status_e mp_login(const Json::Value& args, Json::Value& response);
    http_status_e mp_logout(const Json::Value& args, Json::Value& response);

    std::string _user;
    std::string _token;
};

} // namespace FLECS

#endif // CBE91B30_CC56_4E6A_969C_9A84233C1DA4
