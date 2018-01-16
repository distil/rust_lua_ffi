local example = require('rust-example')

local a = example.make_a("Bilbo Baggins", 111)
print("a", a.string, a.integer)

local b_nil = example.make_b(nil, nil)
print("b_nil", b_nil.string, b_nil.integer)

local b = example.make_b("Frodo Baggins", 0)
print("b", b.string, b.integer)

local c_empty = example.make_c(nil, {})
print("c_empty")
if c_empty.a ~= nil then
    print(c_empty.a.string, c_empty.a.integer)
end
print(c_empty.b.string, c_empty.b.integer)

local c = example.make_c(a, { b })
io.write("c")
if c.a ~= nil then
    io.write(c.a.string, c.a.integer)
end
print()
print(c.b.string, c.b.integer)

local d = example.make_d({ 1, 2, 3, 4, 5, 6 })
for _, value in ipairs(d.integers) do
    io.write(tostring(value).." ")
end
print()

local e_nil = example.make_e(nil, { nil })
io.write("e_nil".." "..tostring(e_nil.integers).." ")
for _, value in ipairs(e_nil.ds) do
    io.write(tostring(value).." ")
end
print()

local e = example.make_e(nil, { d })
io.write("e".." "..tostring(e.integers).." ")
for _, d in ipairs(e.ds) do
    for _, value in ipairs(d.integers) do
        io.write(tostring(value).." ")
    end
    print()
end

local random_short = example.random_short()
print("random_short", random_short)

-- Commented out due to bug in Lua FFI on Mac causing anything to print to stderr/stdout in native to cause a segfault
--local status, msg = pcall(example.i_like_to_panic)
--print("i_like_to_panic", status, msg)
