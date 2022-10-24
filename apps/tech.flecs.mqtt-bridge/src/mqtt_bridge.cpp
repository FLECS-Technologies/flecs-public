
#include "mqtt_bridge.h"

#include <mqtt_protocol.h>
#include <unistd.h>

#include <cstdio>
#include <thread>

#include "util/cxx20/string.h"
#include "util/signal_handler/signal_handler.h"

namespace FLECS {

__attribute__((constructor)) void mqtt_bridge_init()
{
    mosquitto_lib_init();
}

__attribute__((destructor)) void mqtt_bridge_destroy()
{
    mosquitto_lib_cleanup();
}

mqtt_bridge_t::mqtt_bridge_t() noexcept
    : _mosq{mosquitto_new(nullptr, true, this)}
    , _mqtt_connected{}
    , _flunder_client{std::make_unique<flunder_client_t>()}
    , _mqtt_thread{}
    , _flunder_thread{}
{}

mqtt_bridge_t::mqtt_bridge_t(mqtt_bridge_t&& other) noexcept
    : mqtt_bridge_t{}
{
    swap(*this, other);
}

mqtt_bridge_t& mqtt_bridge_t::operator=(mqtt_bridge_t&& other) noexcept
{
    swap(*this, other);
    return *this;
}

mqtt_bridge_t::~mqtt_bridge_t()
{
    mosquitto_destroy(_mosq);
}

auto swap(mqtt_bridge_t& lhs, mqtt_bridge_t& rhs) noexcept //
    -> void
{
    using std::swap;
    swap(lhs._mosq, rhs._mosq);
    swap(lhs._flunder_client, rhs._flunder_client);
    swap(lhs._mqtt_thread, rhs._mqtt_thread);
    swap(lhs._flunder_thread, rhs._flunder_thread);
}

auto mqtt_bridge_t::exec() //
    -> int
{
    _mqtt_thread = std::thread{&mqtt_bridge_t::mqtt_loop, this};
    _flunder_thread = std::thread{&mqtt_bridge_t::flunder_loop, this};

    _flunder_thread.join();
    _mqtt_thread.join();

    return 0;
}

template <typename Func, typename Handle, typename... Args>
auto connect(std::string_view proto, Func f, Handle* h, Args&&... args) //
    -> void
{
    std::fprintf(stdout, "Connecting to %s...\n", proto.data());
    while (!g_stop && (std::invoke(f, h, std::forward<Args>(args)...)) != 0)
    {
        std::fprintf(stderr, "Could not connect to %s - retrying in 2 seconds\n", proto.data());
        sleep(2);
    }
    std::fprintf(stdout, "Connected to %s\n", proto.data());
}

auto mqtt_bridge_t::mqtt_loop() //
    -> void
{
    mosquitto_int_option(_mosq, MOSQ_OPT_PROTOCOL_VERSION, MQTT_PROTOCOL_V5);
    mosquitto_connect_callback_set(_mosq, mosquitto_connect_callback);
    mosquitto_disconnect_callback_set(_mosq, mosquitto_disconnect_callback);
    mosquitto_message_callback_set(_mosq, mosquitto_receive_callback);
    mosquitto_loop_start(_mosq);
    do
    {
        _mqtt_connected = true;
        connect("mqtt", &mosquitto_connect, _mosq, "flecs-mqtt", 1883, 60);

        int sub_options = 0;
        sub_options |= MQTT_SUB_OPT_NO_LOCAL;
        mosquitto_subscribe_v5(_mosq, nullptr, "#", 1, sub_options, nullptr);

        while (!g_stop && _mqtt_connected)
        {
            std::this_thread::sleep_for(std::chrono::milliseconds(500));
        }

        mosquitto_unsubscribe(_mosq, nullptr, "#");
        mosquitto_disconnect(_mosq);
    } while (!g_stop);
}

auto mqtt_bridge_t::flunder_loop() //
    -> void
{
    do
    {
        connect(
            "flunder",
            (int(flunder_client_t::*)(std::string_view, int))(&flunder_client_t::connect),
            _flunder_client.get(),
            FLUNDER_HOST,
            FLUNDER_PORT);

        _flunder_client->subscribe("**", flunder_receive_callback, this);

        while (!g_stop && _flunder_client->is_connected())
        {
            std::this_thread::sleep_for(std::chrono::milliseconds(500));
        };

        _flunder_client->unsubscribe("**");
        _flunder_client->disconnect();
    } while (!g_stop);
}

auto mqtt_bridge_t::flunder_receive_callback(
    flunder_client_t* flunder_client, const flunder_variable_t* var, const void* userp) //
    -> void
{
    if (cxx20::starts_with(var->topic(), "/@"))
    {
        std::fprintf(stdout, "-- dropping message %s due to topic\n", var->topic().data());
        return;
    }
    if (var->encoding() == "application/mqtt-forwarded")
    {
        std::fprintf(
            stdout,
            "-- dropping message %s due to encoding %s\n",
            var->topic().data(),
            var->encoding().data());
        return;
    }

    decltype(auto) mqtt_bridge = const_cast<mqtt_bridge_t*>(static_cast<const mqtt_bridge_t*>(userp));
    if (!mqtt_bridge->mqtt_connected())
    {
        std::fprintf(stdout, "-- dropping flunder message %s as mqtt is not connected\n", var->topic().data());
        return;
    }

    flunder_client->add_mem_storage("flecs-mqtt-bridge", "**");

    mosquitto_publish_v5(
        mqtt_bridge->mosq(),
        nullptr,
        var->topic().data(),
        static_cast<int>(var->len()),
        static_cast<const void*>(var->value().data()),
        1,
        false,
        nullptr);
    std::fprintf(stdout, "++ forwarded flunder message for topic %s to mqtt\n", var->topic().data());
}

auto mqtt_bridge_t::mosquitto_connect_callback(mosquitto*, void* userp, int rc) //
    -> void
{
    decltype(auto) mqtt_bridge = static_cast<mqtt_bridge_t*>(userp);
    mqtt_bridge->_mqtt_connected = (rc == MOSQ_ERR_SUCCESS);
}

auto mqtt_bridge_t::mosquitto_disconnect_callback(mosquitto*, void* userp, int) //
    -> void
{
    decltype(auto) mqtt_bridge = static_cast<mqtt_bridge_t*>(userp);
    mqtt_bridge->_mqtt_connected = false;
}

void mqtt_bridge_t::mosquitto_receive_callback(mosquitto*, void* userp, const mosquitto_message* msg)
{
    decltype(auto) mqtt_bridge = static_cast<mqtt_bridge_t*>(userp);

    if (!mqtt_bridge->flunder_client().is_connected())
    {
        std::fprintf(stdout, "-- dropping mqtt message %s as flunder is not connected\n", msg->topic);
        return;
    }

    mqtt_bridge->flunder_client().add_mem_storage("flecs-mqtt-bridge", "**");
    mqtt_bridge->flunder_client().publish(
        static_cast<std::string_view>(msg->topic),
        static_cast<const void*>(msg->payload),
        static_cast<std::size_t>(msg->payloadlen),
        "application/mqtt-forwarded");

    std::fprintf(stdout, "++ forwarded mqtt message for topic %s to flunder\n", msg->topic);
}

} // namespace FLECS

int main()
{
    FLECS::signal_handler_init();

    auto bridge = FLECS::mqtt_bridge_t{};

    return bridge.exec();
}
