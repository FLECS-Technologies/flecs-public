// Copyright 2021 FLECS Technologies GmbH
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

#include "service/service_rpc.h"
#include "util/container/map_constexpr.h"

#include <iostream>
#include <list>
#include <memory>
#include <regex>
#include <string>

namespace FLECS
{

int service_rpc::do_process(int /*argc*/, char** /*argv*/)
{
    return 0;
}
/*
void emplace_arg(std::list<FLECS::any>& args, const char* arg)
{
    const std::regex arg_type { "\\((\\w+)\\)(.*)" };
    std::cmatch regex_match;
    if (!std::regex_match(arg, regex_match, arg_type) || regex_match.length() < 2)
    {
        return;
    }
    std::string type = regex_match[1].str();
    if (type == "string")
    {
        args.emplace_back(any(regex_match[2].str()));
    }
    else if (type == "bool")
    {
        bool val = regex_match[2].str() == "true" ?
            true :
            false;
        args.emplace_back(any(&val));
    }
}

int service_rpc(int argc, char** argv)
{
    for (int i = 0; i < argc; ++i)
    {
        std::cout << "Argument " << i << ": " << argv[i] << std::endl;
    }

    if (argc < 3)
    {
        return 1;
    }

    std::string action = argv[0];
    std::string callee = argv[1];
    std::string method = argv[2];
    std::list<FLECS::any> args;

    size_t i = 3;
    while (argv[i] != nullptr)
    {
        emplace_arg(args, argv[i]);
        ++i;
    }

    std::cout << "Action: " << action;
    std::cout << "\nCallee: " << callee;
    std::cout << "\nMethod: " << method;
    std::cout << std::endl;

    std::shared_ptr<Message> msg_p = std::make_shared<RPCCallReq>(MessageHeader { 0, EOK });
    RPCCallReq& req_msg = static_cast<RPCCallReq&>(*msg_p);

    req_msg.Body()._callee = argv[1];
    req_msg.Body()._method = argv[2];
    req_msg.Body()._arguments = args;

    FLECS::serialized_t ser;
    ser << req_msg;

    void* zmq_ctx = zmq_ctx_new();
    void* zmq_sock = zmq_socket(zmq_ctx, ZMQ_REQ);
    zmq_connect(zmq_sock, "tcp://127.0.0.1:8960");
    zmq_send(zmq_sock, ser.data(), ser.size(), 0);
    (void)zmq_recv(zmq_sock, ser.data(), ser.size(), 0);

    return 0;
}
*/
} // namespace FLECS
