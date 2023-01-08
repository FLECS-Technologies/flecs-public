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

#pragma once

#include <cstdint>
#include <mutex>
#include <string>

#include "job_id.h"

namespace FLECS {

class job_progress_t
{
public:
    explicit job_progress_t(job_id_t job_id);

    auto job_id() const noexcept //
        -> job_id_t;

private:
    job_id_t _job_id; /** unique job id */

    std::uint32_t status;   /** @todo job status - replace by enum */
    std::string desc;       /** job description (e.g. "Install app xyz (123) ")*/
    std::int16_t num_steps; /** total number of steps  */
    struct
    {
        std::string desc;        /** current step description*/
        std::int16_t num;        /** number of current step */
        std::string unit;        /** unit of current step's operation (e.g. "B" when downloading)*/
        std::size_t units_total; /** total units to process */
        std::size_t units_done;  /** processed units so far*/
        std::size_t rate;        /** processing rate in units per second*/
    } current_step;              /** current step't meta info*/
    bool done;                   /** true after job finished */
    struct
    {
        std::int32_t code;    /** exit code */
        std::string message;  /** error/success message*/
        std::string location; /** where a newly created resource can be found (e.g. /instances/abcd1234) */
    } result;
};

auto operator<(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;
auto operator<=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;
auto operator>(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;
auto operator>=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;
auto operator==(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;
auto operator!=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;

} // namespace  FLECS
