Finish
======

Those tests check the return codes and the behavior of `llhttp_finish()` C API.

## It should be safe to finish with cb after empty response

<!-- meta={"type": "response-finish"} -->
```http
HTTP/1.1 200 OK


```

```log
off=0 message begin
off=13 len=2 span[status]="OK"
off=19 headers complete status=200 v=1/1 flags=0 content_length=0
off=NULL finish=1
```
