-- cargo run -- examples/json.lua

dayax:get('/api/person/:id', function (req) 
    return {
        id = req.path.id,
        age = 10,
        name = "Manolo"
    }
end)

dayax:delete('/api/person/:id', function (req) 
    return {
        msg = "Deleted person with name" .. req.path.id,
        id = req.path.id
    }
end)


