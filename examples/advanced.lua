dayax:get("/", function(req) 
    local body = req.searchParams.format == 'json' and req or "Json request are more interesting"
    return {
        headers = {
            ["X-my-custom-header"] = "FizzBuz"
        },
        statusCode = 418,
        body = body
    }
end)

dayax:get("/redirect", function (req) 
    return { redirect = "/" }
end)

dayax:any("/echo/:hello", function (req) return { body = req } end)