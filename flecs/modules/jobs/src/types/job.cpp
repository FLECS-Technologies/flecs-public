// Copyright 2021-2023 FLECS Technologies GmbH
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

#include "flecs/modules/jobs/types/job.h"

namespace flecs {
namespace jobs {

job_t::job_t(jobs::job_t::callable_t callable)
    : _callable{std::move(callable)}
{}

auto jobs::job_t::callable() const noexcept //
    -> const callable_t&
{
    return _callable;
}

} // namespace jobs
} // namespace flecs
