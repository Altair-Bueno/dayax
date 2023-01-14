dayax:get("/", function(req) 
    local body = req.searchParams.format == 'json' and { response = "json"} or "String response"
    return {
        headers = {
            ["X-my-custom-header"] = "FizzBuz"
        },
        statusCode = 418,
        body = body
    }
end)

dayax:get("/redirect", function (req) 
    return { redirect = "/"}
end)