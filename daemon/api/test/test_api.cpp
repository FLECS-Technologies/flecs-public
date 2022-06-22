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

#include <cpr/cpr.h>

#include <thread>

#include "daemon/api/api.h"
#include "daemon/api/endpoints/endpoints.h"
#include "gtest/gtest.h"
#include "util/cxx20/string.h"

class test_flecs_api_t
{
public:
    test_flecs_api_t() { std::thread{&FLECS::flecs_api_t::run, &_api}.detach(); }

private:
    FLECS::flecs_api_t _api;
};

FLECS::http_status_e test_endpoint_get(const FLECS::json_t&, FLECS::json_t& response)
{
    response["additionalInfo"] = "OK";
    return FLECS::http_status_e::Ok;
}

FLECS::http_status_e test_endpoint_post(const FLECS::json_t&, FLECS::json_t& response)
{
    response["additionalInfo"] = "OK";

    return FLECS::http_status_e::Ok;
}

static auto test_api = test_flecs_api_t{};

TEST(api, unix_socket_not_implemented)
{
    const auto res = cpr::Get(cpr::Url{"http:/flecs/test/get"}, cpr::UnixSocket{"flecs.sock"});

    ASSERT_EQ(static_cast<FLECS::http_status_e>(res.status_code), FLECS::http_status_e::NotImplemented);
}

TEST(api, tcp_socket_not_implemented)
{
    const auto res = cpr::Get(cpr::Url{"http://localhost:8951/test/get"});

    ASSERT_EQ(static_cast<FLECS::http_status_e>(res.status_code), FLECS::http_status_e::NotImplemented);
}

TEST(api, endpoint_get)
{
    FLECS::api::register_endpoint("/test/get", llhttp_method::HTTP_GET, &test_endpoint_get);

    const auto res = cpr::Get(cpr::Url{"http://localhost:8951/test/get"});

    ASSERT_EQ(static_cast<FLECS::http_status_e>(res.status_code), FLECS::http_status_e::Ok);
    ASSERT_EQ(res.text, R"({"additionalInfo":"OK"})");
}

TEST(api, endpoint_post)
{
    FLECS::api::register_endpoint("/test/post", llhttp_method::HTTP_POST, &test_endpoint_post);

    const auto json = FLECS::json_t({"arg", "value"});
    const auto res = cpr::Post(cpr::Url{"http://localhost:8951/test/post"}, cpr::Body{json.dump()});

    ASSERT_EQ(static_cast<FLECS::http_status_e>(res.status_code), FLECS::http_status_e::Ok);
    ASSERT_EQ(res.text, R"({"additionalInfo":"OK"})");
}

TEST(api, bad_request_1)
{
    auto sock_fd = socket(AF_INET, SOCK_STREAM, 0);
    ASSERT_NE(sock_fd, -1);

    const auto addr =
        sockaddr_in{.sin_family = AF_INET, .sin_port = htons(8951), {.s_addr = 0x0100007f}, .sin_zero = {}};
    auto conn_res = connect(sock_fd, reinterpret_cast<const sockaddr*>(&addr), sizeof(addr));
    ASSERT_NE(conn_res, -1);

    auto buf = std::unique_ptr<char[]>{new char[1024]};
    auto sent = send(sock_fd, buf.get(), 1024, 0);
    ASSERT_NE(sent, -1);

    auto rc = recv(sock_fd, buf.get(), 1024, 0);
    ASSERT_NE(rc, -1);

    ASSERT_TRUE(FLECS::cxx20::starts_with(buf.get(), "HTTP/1.1 400 Bad Request"));

    close(sock_fd);
}

TEST(api, bad_request_2)
{
    using std::operator""sv;

    auto sock_fd = socket(AF_INET, SOCK_STREAM, 0);
    ASSERT_NE(sock_fd, -1);

    const auto addr =
        sockaddr_in{.sin_family = AF_INET, .sin_port = htons(8951), {.s_addr = 0x0100007f}, .sin_zero = {}};
    auto conn_res = connect(sock_fd, reinterpret_cast<const sockaddr*>(&addr), sizeof(addr));
    ASSERT_NE(conn_res, -1);

    auto buf = std::unique_ptr<char[]>{new char[1024]};
    const auto req =
        "POST /test/post HTTP/1.1\r\n"           //
        "Host: localhost:8951\r\n"               //
        "Content-Length: 121\r\n"                //
        "Content-Type: application/json\r\n\r\n" //
        "Not a JSON body\r\n"sv;

    std::memset(buf.get(), 0, 1024);
    std::copy(std::begin(req), std::end(req), buf.get());

    auto sent = send(sock_fd, buf.get(), req.size(), 0);
    ASSERT_NE(sent, -1);

    auto rc = recv(sock_fd, buf.get(), 1024, 0);
    ASSERT_NE(rc, -1);

    ASSERT_TRUE(FLECS::cxx20::starts_with(buf.get(), "HTTP/1.1 400 Bad Request"));
}

TEST(api, not_allowed)
{
    const auto res = cpr::Patch(cpr::Url{"http://localhost:8951/test/patch"});

    ASSERT_EQ(static_cast<FLECS::http_status_e>(res.status_code), FLECS::http_status_e::MethodNotAllowed);
}
