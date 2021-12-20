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

#ifndef FLECS_util_curl_easy_ext_h
#define FLECS_util_curl_easy_ext_h

#include <string>

#include "curl/curl.h"

namespace FLECS {

class curl_easy_ext
{
public:
    curl_easy_ext();

    ~curl_easy_ext();

    template <CURLoption option, typename T>
    CURLcode setopt(T param);

    CURLcode perform();

    operator bool() const noexcept;

private:
    template <typename T>
    CURLcode setopt(CURLoption option, T param);

    CURL* _curl;

    int _write_fd;
};

template <typename T>
CURLcode curl_easy_ext::setopt(CURLoption option, T param)
{
    if (!_curl)
    {
        return CURLE_FAILED_INIT;
    }
    return curl_easy_setopt(_curl, option, param);
}

#define CURL_EASY_EXT_SETOPT(option, type)                    \
    template <>                                               \
    inline CURLcode curl_easy_ext::setopt<option>(type param) \
    {                                                         \
        return setopt(option, param);                         \
    }

CURL_EASY_EXT_SETOPT(CURLOPT_FAILONERROR, long);
CURL_EASY_EXT_SETOPT(CURLOPT_NOPROGRESS, long);
CURL_EASY_EXT_SETOPT(CURLOPT_PORT, long);
CURL_EASY_EXT_SETOPT(CURLOPT_URL, const char*);
CURL_EASY_EXT_SETOPT(CURLOPT_VERBOSE, long);
CURL_EASY_EXT_SETOPT(CURLOPT_WRITEDATA, void*);
CURL_EASY_EXT_SETOPT(CURLOPT_WRITEFUNCTION, curl_write_callback);

size_t curl_easy_ext_write_cb(char* buffer, size_t size, size_t nmemb, void* userp);

} // namespace FLECS

#endif // FLECS_util_curl_easy_ext_h
