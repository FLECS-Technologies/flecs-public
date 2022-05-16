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

#ifndef ED9FB390_4279_4DD1_9325_B7D3F8F86D98
#define ED9FB390_4279_4DD1_9325_B7D3F8F86D98

#include "data_layer/data_layer.h"
#include "flunder/flunder_client.h"

namespace FLECS {
namespace Private {

class module_data_layer_private_t
{
public:
    module_data_layer_private_t();
    ~module_data_layer_private_t();

    http_status_e do_browse(const std::string& path, json_t& response);

private:
    flunder_client_t _client;
};

} // namespace Private
} // namespace FLECS

#endif // ED9FB390_4279_4DD1_9325_B7D3F8F86D98
