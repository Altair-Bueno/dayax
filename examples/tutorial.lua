--[[
# Dayax tutorial

Execute this file using the following command: 

```sh
RUST_LOG=dayax=debug cargo run -- examples/tutorial.lua
```
]]

--[[
# Hello world
curl localhost:8000/
]]
dayax:get("/", function() return "Hello world" end)

--[[
# Sending redirects
curl localhost:8000/redirect -v
]]
dayax:get("/redirect", function (req) 
    return { redirect = "/" }
end)

--[[
# Dayax parses JSON and form encoded data automatically based on the 
# `Content-Type` header. Everything else is treated as UTF-8 strings.
curl 'localhost:8000/echo/juan?pages=1' -v \
    -d form_data=10

curl 'localhost:8000/echo/juan?pages=1' -v \
    -H 'Content-Type: application/json' \
    -d '{"hello": "world"}'
]]
dayax:any("/echo/:name", function (req) 
    return { 
        -- Any table provided on the `body` key will be treated as JSON. 
        -- Strings will be treated as `text/plain`
        body = req,
        -- Custom headers added to the response
        headers = {
            ["X-my-custom-header"] = "FizzBuz"
        },
        -- Response status code
        statusCode = 418
    } 
end)

--[[
# Empty responses are also allowed
curl localhost:8000/empty -v
]]
dayax:post("/empty", function() end)
