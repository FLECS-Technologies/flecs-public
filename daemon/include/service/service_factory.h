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

#ifndef FLECS_service_service_factory_h
#define FLECS_service_service_factory_h

#include <memory>

namespace FLECS {

class service_t;

template <typename T, typename... Args>
auto make_service(Args&&... args)
{
    return std::shared_ptr<service_t>{std::make_shared<T>(args...)};
}

} // namespace FLECS

#endif // FLECS_service_service_factory_h
