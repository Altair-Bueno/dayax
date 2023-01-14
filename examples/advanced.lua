dayax:get("/", function(req) 
    return {
        headers = {
            ["X-my-custom-header"] = "FizzBuz"
        },
        body = { response= "OK" }
    }
end)

dayax:get("/redirect", function (req) 
    return { redirect = "/"}
end)