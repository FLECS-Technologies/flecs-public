#include "ui/backend/http_request_handler.h"

#include "util/literals.h"
#include "util/llhttp_ext/llhttp_ext.h"
#include "util/map_constexpr.h"
#include "util/socket/tcp_server.h"

#include <csignal>
#include <iostream>
#include <thread>

bool g_stop = false;

int http_request_handler_thread(FLECS::tcp_socket_t&& conn_socket)
{
    FLECS::http_request_handler_t handler { std::move(conn_socket) };
    auto err = handler.dispatch();
    auto res = handler.send_response(err);
    if (res <= 0)
    {
        return 1;
    }
    return 0;
}

void signal_handler(int)
{
    g_stop = true;
}

int main()
{
    struct sigaction signal_action {};
    signal_action.sa_handler = &signal_handler;
    sigaction(SIGTERM, &signal_action, nullptr);
    sigaction(SIGINT, &signal_action, nullptr);

    FLECS::tcp_server_t server { 42000, INADDR_LOOPBACK, 10 };
    if (!server.is_running())
    {
        return 1;
    }

    do
    {
        FLECS::tcp_socket_t conn_socket = server.accept(nullptr, nullptr);
        if (conn_socket.is_valid())
        {
            std::thread handle_thread { &http_request_handler_thread, std::move(conn_socket) };
            handle_thread.detach();
        }
    } while (!g_stop);
}
