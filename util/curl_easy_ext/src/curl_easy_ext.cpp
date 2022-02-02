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

#include "curl_easy_ext.h"

#include <unistd.h>

#include <iostream>

namespace FLECS {

curl_easy_ext::curl_easy_ext(const std::string& url, int* write_fd)
    : _curl{}
{
    if (!_ref_count)
    {
        auto lock = std::lock_guard<std::mutex>{_ref_mutex};
        curl_global_init(CURL_GLOBAL_DEFAULT);
        ++_ref_count;
    }

    _curl = curl_easy_init();
    if (_curl)
    {
        setopt<CURLOPT_WRITEFUNCTION>(&curl_easy_ext_write_cb);
        setopt<CURLOPT_FAILONERROR>(1L);
        setopt<CURLOPT_URL>(url.c_str());
        setopt<CURLOPT_WRITEDATA>(static_cast<void*>(write_fd));
    }
}

curl_easy_ext::~curl_easy_ext()
{
    curl_easy_cleanup(_curl);
    auto lock = std::lock_guard<std::mutex>{_ref_mutex};
    --_ref_count;
    if (!_ref_count)
    {
        curl_global_cleanup();
    }
}

curl_easy_ext::operator bool() const noexcept
{
    return _curl != nullptr;
}

CURLcode curl_easy_ext::perform()
{
    return curl_easy_perform(_curl);
}

size_t curl_easy_ext_write_cb(char* buffer, size_t size, size_t nmemb, void* userp)
{
    int fd = *reinterpret_cast<int*>(userp);
    ssize_t res = write(fd, buffer, size * nmemb);
    if (res != static_cast<ssize_t>(size * nmemb))
    {
        std::cerr << "Wrote " << res << " of " << (size * nmemb) << " items to " << fd << std::endl;
        return 0;
    }
    return res;
}

} // namespace FLECS
