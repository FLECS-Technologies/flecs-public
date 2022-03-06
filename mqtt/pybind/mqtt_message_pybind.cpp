#include <pybind11/pybind11.h>

#include "mqtt/mqtt_message.h"

namespace py = pybind11;

PYBIND11_MODULE(mqtt_message, m)
{
    py::class_<FLECS::mqtt_message_t>(m, "mqtt_message")
        .def(py::init())
        .def_readwrite("id", &FLECS::mqtt_message_t::id)
        .def_readwrite("topic", &FLECS::mqtt_message_t::topic)
        .def_readwrite("payload", &FLECS::mqtt_message_t::payload)
        .def_readwrite("payloadlen", &FLECS::mqtt_message_t::payloadlen)
        .def_readwrite("qos", &FLECS::mqtt_message_t::qos)
        .def_readwrite("retain", &FLECS::mqtt_message_t::retain);
}
