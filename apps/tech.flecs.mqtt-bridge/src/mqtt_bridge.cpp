
#include "mqtt_bridge.h"

#include <cstdio>
#include <thread>

#include "util/signal_handler/signal_handler.h"

namespace FLECS {

mqtt_bridge_t::mqtt_bridge_t() noexcept
    : _mqtt_connected{}
    , _flunder_connected{}
    , _mqtt_client{}
    , _flunder_client{}
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
{}

void swap(mqtt_bridge_t& lhs, mqtt_bridge_t& rhs) noexcept
{
    using std::swap;
    swap(lhs._mqtt_client, rhs._mqtt_client);
    swap(lhs._flunder_client, rhs._flunder_client);
}

int mqtt_bridge_t::loop()
{
    do
    {
        if (!mqtt_connected())
        {
            connect_mqtt();
        }
        if (!flunder_connected())
        {
            connect_flunder();
        }

        while (mqtt_connected() && flunder_connected() && !g_stop)
        {
#if 0
            expire_values();
#endif // 0
            std::this_thread::sleep_for(std::chrono::seconds(1));
        }
    } while (!g_stop);

    disconnect_mqtt();
    disconnect_flunder();

    return 0;
}

#if 0
void mqtt_bridge_t::expire_values()
{
    auto res = _flunder_client.get("/**");
    if (std::get<0>(res) != 0)
    {
        return;
    }
    decltype(auto) values = std::get<1>(res);
    for (const auto& value : values)
    {
        const auto now = std::chrono::system_clock().now().time_since_epoch().count();
    }
}
#endif // 0

void mqtt_bridge_t::connect_mqtt()
{
    auto res = _mqtt_client.connect(FLECS::MQTT_HOST, FLECS::MQTT_PORT, FLECS::MQTT_KEEPALIVE);
    while (res != FLECS::MQTT_ERR_OK)
    {
        std::fprintf(
            stderr,
            "Could not connect to MQTT host %s:%d - retrying in 10 seconds\n",
            FLECS::MQTT_HOST,
            FLECS::MQTT_PORT);
        std::this_thread::sleep_for(std::chrono::seconds(10));
        res = _mqtt_client.connect(FLECS::MQTT_HOST, FLECS::MQTT_PORT, FLECS::MQTT_KEEPALIVE);
    }
    std::fprintf(stdout, "Connected to MQTT broker\n");

    _mqtt_client.receive_callback_set(&mqtt_receive_callback, this);
    _mqtt_client.disconnect_callback_set(&mqtt_disconnect_callback, this);
    _mqtt_client.subscribe("#", 0);

    _mqtt_connected = true;
}

void mqtt_bridge_t::connect_flunder()
{
    auto res = _flunder_client.connect(FLECS::FLUNDER_HOST, FLECS::FLUNDER_PORT);
    while (res != 0)
    {
        std::fprintf(
            stderr,
            "Could not connect to flunder host %s:%d - retrying in 10 seconds\n",
            FLECS::FLUNDER_HOST,
            FLECS::FLUNDER_PORT);
        std::this_thread::sleep_for(std::chrono::seconds(10));
        res = _flunder_client.connect(FLECS::FLUNDER_HOST, FLECS::FLUNDER_PORT);
    }
    _flunder_client.add_mem_storage("flecs-mqtt-bridge", "/**");
    std::fprintf(stdout, "Connected to flunder broker\n");

    _flunder_connected = true;
}

void mqtt_bridge_t::disconnect_mqtt()
{
    _mqtt_client = mqtt_client_t{};
}

void mqtt_bridge_t::disconnect_flunder()
{
    _flunder_client = flunder_client_t{};
}

void mqtt_bridge_t::mqtt_receive_callback(FLECS::mqtt_client_t* /*client*/, FLECS::mqtt_message_t* msg, void* userp)
{
    decltype(auto) bridge = *static_cast<FLECS::mqtt_bridge_t*>(userp);
    bridge.flunder_client().publish(msg->topic, msg->payload, msg->payloadlen);
}

void mqtt_bridge_t::mqtt_disconnect_callback(FLECS::mqtt_client_t* /*client*/, void* userp)
{
    decltype(auto) bridge = *static_cast<FLECS::mqtt_bridge_t*>(userp);
    std::fprintf(stdout, "Disconnected from MQTT broker\n");
    bridge.mqtt_connected(false);
}

} // namespace FLECS

int main()
{
    FLECS::signal_handler_init();

    auto bridge = FLECS::mqtt_bridge_t{};

    return bridge.loop();
}
