local luaunit = require('luaunit')


--[[
    Recursively discover all files prefixed with 'test' and load them.

    @param string directory
    @return table tests
--]]

local function discoverTests(directory)
    for testName in io.popen("find "..directory.." -name \"test*.lua\""):lines() do
        print(testName)
        local keyName = testName:sub(string.len(directory) + 2, -5)
        print(keyName)
        local importName = keyName:gsub("/", ".")

        _G[keyName] = require(importName)
    end
end

--[[
    Discovers all tests and runs them
--]]

local function run(entrypoint)
    local lu = luaunit.LuaUnit.new()
    luaunit:setVerbosity(luaunit.VERBOSITY_VERBOSE)

    discoverTests(entrypoint)
    lu:runSuite()

    os.exit((lu.result.failureCount > 0 and 1 or 0)
        + (lu.result.errorCount > 0 and 2 or 0))
end


run("lua/test")
