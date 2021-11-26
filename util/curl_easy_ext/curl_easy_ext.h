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

#include <curl/curl.h>

#include <string>

namespace FLECS {

class curl_easy_ext
{
public:
    curl_easy_ext(const char* url, int write_fd);

    ~curl_easy_ext();

    CURLcode perform();

    operator bool() const noexcept;

private:
    CURL* _curl;

    const char* _url;
    int _write_fd;
};

size_t curl_easy_ext_write_cb(char* buffer, size_t size, size_t nmemb, void* userp);

} // namespace FLECS


#endif // FLECS_util_curl_easy_ext_h
