#include "llhttp_ext.h"

#include <cctype>

namespace FLECS {

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
