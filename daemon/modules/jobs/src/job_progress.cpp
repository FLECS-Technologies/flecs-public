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

#include "job_progress.h"

namespace FLECS {

job_progress_t::job_progress_t(job_id_t job_id)
    : _job_id{job_id}
{}

auto job_progress_t::job_id() const noexcept //
    -> job_id_t
{
    return _job_id;
}

auto operator<(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return lhs.job_id() < rhs.job_id();
}

auto operator<=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return !(lhs > rhs);
}

auto operator>(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return rhs < lhs;
}

auto operator>=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return !(lhs < rhs);
}

auto operator==(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return lhs.job_id() == rhs.job_id();
}

auto operator!=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return !(lhs == rhs);
}

} // namespace FLECS
