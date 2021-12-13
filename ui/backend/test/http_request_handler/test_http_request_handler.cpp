#include <sys/socket.h>
#include <sys/types.h>
#include <sys/un.h>

#include <thread>
#include <utility>

#include "gmock/gmock.h"
#include "gtest/gtest.h"
#include "ui/backend/http_request_handler.h"
#include "util/socket/socket.h"
#include "util/socket/testing/mock_socket.h"

TEST(Test_Backend, tc01_empty_request)
{
    FLECS::http_request_handler_t uut(FLECS::mock_tcp_socket_t{});
    const FLECS::mock_socket_t& mock_socket = static_cast<const FLECS::mock_socket_t&>(uut.conn_socket());
    EXPECT_CALL(mock_socket, recv);
    // uut.dispatch();
}
