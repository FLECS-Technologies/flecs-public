#include "llhttp_ext.h"

#include <cctype>

namespace FLECS {

void llhttp_ext_settings_init(llhttp_settings_t* settings)
{
    llhttp_settings_init(settings);
    settings->on_body = &llhttp_ext_on_body;
    settings->on_url = &llhttp_ext_on_url;
    settings->on_message_complete = &llhttp_ext_on_message_complete;
}

int llhttp_ext_on_body(llhttp_t* llhttp, const char* at, size_t length)
{
    using FLECS::llhttp_ext_t;
    llhttp_ext_t* llhttp_ext = static_cast<llhttp_ext_t*>(llhttp);
    llhttp_ext->_body.append(at, length);
    return 0;
}

int llhttp_ext_on_url(llhttp_t* llhttp, const char* at, size_t length)
{
    using FLECS::llhttp_ext_t;
    llhttp_ext_t* llhttp_ext = static_cast<llhttp_ext_t*>(llhttp);
    llhttp_ext->_url.append(at, length);
    return 0;
}

int llhttp_ext_on_message_complete(llhttp_t* /*llhttp*/)
{
    return 0;
}

} // namespace FLECS
