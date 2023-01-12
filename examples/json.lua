-- cargo run -- examples/json.lua

dayax['/api/person/:name'] = function () 
    return {
        name = "David",
        age = 10
    }
end 
