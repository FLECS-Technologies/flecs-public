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

#ifndef FLECS_daemon_api_h
#define FLECS_daemon_api_h

#include <memory>

#include "util/http/status_codes.h"
#include "util/socket/tcp_server.h"

namespace Json {
class CharReader;
} // namespace Json

namespace FLECS {

/*! API for communication with the outside world. Runs an HTTP server handling requests on registered endpoints.
 */
class flecs_api_t
{
public:
    /*! @brief Default constructor. Initializes TCP server for API requests on Port 8951.
     */
    flecs_api_t();

    ~flecs_api_t();

    /*! @brief Cyclically accepts pending connections and processes a single command
     */
    int run();

private:
    /*! @brief Processes a single command read from a connected client.
     *
     * Receives up to 128kiB of data from the connected socket, parses the request and passes it to the desired
     * endpoint, if available.
     *
     * @return HTTP status code
     *      200: OK - endpoint was found and handled command successfully
     *      400: Bad Request - no or invalid data was received
     *      500: Internal Server Error - an error occurred while the endpoint processed the request
     *      501: Not Implemented - requested endpoint is not available
     */
    http_status_e process(FLECS::tcp_socket_t& conn_socket);

    tcp_server_t _server;
    std::unique_ptr<Json::CharReader> _json_reader;
};

} // namespace FLECS

#endif // FLECS_daemon_api_h
