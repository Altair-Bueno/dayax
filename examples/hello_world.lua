-- cargo run -- examples/hello_world.lua
dayax:get("/", function () 
    return "Hello world!"
end)
