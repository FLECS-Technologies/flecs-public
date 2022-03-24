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

#ifndef ADF3DE5E_D65D_4E99_8318_021E8B92926E
#define ADF3DE5E_D65D_4E99_8318_021E8B92926E

#include <memory>
#include <tuple>

#include "flunder_client.h"

namespace Json {
class CharReader;
} // namespace Json

namespace FLECS {
namespace Private {

class flunder_client_private_t
{
public:
    flunder_client_private_t();
    ~flunder_client_private_t();

    FLECS_EXPORT int connect(std::string_view host, int port);

    FLECS_EXPORT int reconnect();

    FLECS_EXPORT int disconnect();

    FLECS_EXPORT int publish(std::string_view path, const std::string& type, const std::string& value);

    // FLECS_EXPORT int subscribe(std::string_view path, const flunder_client_t::subscribe_callback_t& cbk);
    // FLECS_EXPORT int unsubscribe(std::string_view path);

    FLECS_EXPORT auto get(std::string_view path) -> std::tuple<int, std::vector<flunder_variable_t>>;
    FLECS_EXPORT int erase(std::string_view path);

private:
    std::unique_ptr<Json::CharReader> _json_reader;
};

} // namespace Private
} // namespace FLECS

#endif // ADF3DE5E_D65D_4E99_8318_021E8B92926E
