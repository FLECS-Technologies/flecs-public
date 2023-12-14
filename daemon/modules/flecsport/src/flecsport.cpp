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

#include "daemon/modules/flecsport/flecsport.h"

#include "daemon/modules/instances/types/instance_id.h"
#include "daemon/modules/apps/types/app_key.h"
#include "daemon/modules/factory/factory.h"
#include "daemon/modules/flecsport/impl/flecsport_impl.h"
#include "util/datetime/datetime.h"

namespace flecs {
namespace module {

namespace {
register_module_t<flecsport_t> _reg("flecsport");
}

flecsport_t::flecsport_t()
    : _impl{new impl::flecsport_t{this}}
{}

flecsport_t::~flecsport_t()
{}

auto flecsport_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/exports").methods("GET"_method)([this]() { return http_list(); });

    FLECS_V2_ROUTE("/exports/<string>").methods("GET"_method)([this](const std::string& export_id) {
        return http_download(export_id);
    });

    FLECS_V2_ROUTE("/exports/<string>").methods("DELETE"_method)([this](const std::string& export_id) {
        return http_remove(export_id);
    });

    FLECS_V2_ROUTE("/exports/create").methods("POST"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        auto apps = std::vector<apps::key_t>{};
        if (args.contains("apps")) {
            args["apps"].get_to(apps);
        }
        auto instances = std::vector<instances::id_t>{};
        if (args.contains("instances")) {
            for (const auto& instance_id : args["instances"]) {
                instances.emplace_back(instances::id_t{instance_id.get<std::string_view>()});
            }
        }
        if (apps.empty() && instances.empty()) {
            return crow::response{crow::status::BAD_REQUEST};
        }
        return http_export_to(std::move(apps), std::move(instances));
    });

    FLECS_V2_ROUTE("/imports").methods("POST"_method)([this](const crow::request& req) {
        const auto it = req.headers.find("X-Uploaded-Filename");
        if (it == req.headers.cend()) {
            return crow::response{
                crow::status::BAD_REQUEST,
                "json",
                "{\"additionalInfo\":\"Missing header X-Uploaded-Filename in request\"}"};
        }
        return http_import_from(fs::path{"/var/lib/flecs/imports"} / it->second);
    });

    return _impl->do_init();
}

auto flecsport_t::http_list() //
    -> crow::response
{
    auto res = json_t::array();
    auto exports = _impl->do_exports();

    for (auto& e : exports) {
        res.push_back(e);
    }

    return {crow::status::OK, "json", res.dump()};
}

auto flecsport_t::http_download(const std::string& export_id) //
    -> crow::response
{
    const auto export_filename = export_id + ".tar.gz";
    const auto export_path = fs::path{"/var/lib/flecs/exports"} / export_filename;

    if (fs::is_regular_file(export_path)) {
        auto res = crow::response{};
        res.set_static_file_info_unsafe(export_path.string());
        res.set_header("Content-Type", "application/gzip");
        res.set_header("Content-Disposition", "attachment; filename=\"" + export_filename + "\"");
        return res;
    }

    return crow::response{crow::status::NOT_FOUND};
}

auto flecsport_t::http_remove(const std::string& export_id) //
    -> crow::response
{
    const auto archive = fs::path{"/var/lib/flecs/exports"} / (export_id + ".tar.gz");

    auto ec = std::error_code{};
    if (!fs::is_regular_file(archive)) {
        return crow::response{crow::status::NOT_FOUND};
    }

    fs::remove(archive, ec);
    if (ec) {
        return crow::response{crow::status::INTERNAL_SERVER_ERROR};
    }

    return crow::response{crow::status::OK};
}

auto flecsport_t::http_export_to(std::vector<apps::key_t> apps, std::vector<instances::id_t> instances) //
    -> crow::response
{
    const auto now = unix_time(precision_e::milliseconds);
    auto dest_dir = fs::path{"/var/lib/flecs/exports"} / now;
    auto job_id = _impl->queue_export_to(std::move(apps), std::move(instances), std::move(dest_dir));
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto flecsport_t::http_import_from(std::string archive) //
    -> crow::response
{
    auto job_id = _impl->queue_import_from(std::move(archive));
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

} // namespace module
} // namespace flecs
