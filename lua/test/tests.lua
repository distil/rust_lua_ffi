local luaunit = require('luaunit')
local unit = require('rust-unit')

local M = {}

function M.testIntegers()
    luaunit.assertEquals(unit.square_i8(2), 2 * 2)
    luaunit.assertEquals(unit.square_i16(3), 3 * 3)
    luaunit.assertEquals(unit.square_i32(4), 4 * 4)
    luaunit.assertEquals(unit.square_u8(2), 2 * 2)
    luaunit.assertEquals(unit.square_u16(3), 3 * 3)
    luaunit.assertEquals(unit.square_u32(4), 4 * 4)
    luaunit.assertAlmostEquals(unit.square_f64(5.3), 5.3 * 5.3)
end

function M.testConcatenateStrings()
    luaunit.assertEquals(unit.concatenate_strings("Bilbo", "Baggins", " "), "Bilbo Baggins")
    luaunit.assertEquals(unit.concatenate_strings("", "", ""), "")
end

function M.testConcatenateSlices()
    luaunit.assertEquals(
        unit.concatenate_u16_slices(
            { 1, 2, 3, 4 },
            { 5, 6, 7 }),
        { 1, 2, 3, 4, 5, 6, 7 })
    luaunit.assertEquals(
        unit.concatenate_u16_slices(
            { 1, 2, 3, 4 },
            {}),
        { 1, 2, 3, 4 })
    luaunit.assertEquals(unit.concatenate_u16_slices({}, {}), {})
end

function M.testConcatenateStructs()
    local a = unit.concatenate_a(unit.make_a("Dead", 0xDEAD), unit.make_a("beef", 0xBEEF), " ")
    luaunit.assertEquals(a.string, "Dead beef")
    luaunit.assertEquals(a.integer, 0xDEAD + 0xBEEF)
end

function M.testConcatenateVecs()
    luaunit.assertEquals(
        unit.concatenate_vec_i32({ 1, 2, 3, 4 }, { 5, 6, 7 }),
        { 1, 2, 3, 4, 5, 6, 7 })
    luaunit.assertEquals(
        unit.concatenate_vec_string({"Red", "Green"}, {"Blue"}),
        { "Red", "Green", "Blue" })
    luaunit.assertEquals(
        unit.concatenate_vec_vec_i32({ { 1, 2, 3 }, { 4, 5 } }, { { 6 }, {} }),
        { { 1, 2, 3 }, { 4, 5 }, { 6 }, {} })
    luaunit.assertEquals(
        unit.concatenate_vec_vec_string({ {}, {"Red", "Green"}, {"Blue"} }, { {}, {} }),
        { {}, {"Red", "Green"}, {"Blue"}, {}, {} })
end

function M.testOptionOr()
    luaunit.assertEquals(unit.option_i32_or(nil, nil), nil)
    luaunit.assertEquals(unit.option_i32_or(42, nil), 42)
    luaunit.assertEquals(unit.option_i32_or(nil, 42), 42)
    luaunit.assertEquals(unit.option_i32_or(42, 43), 42)

    luaunit.assertEquals(unit.option_string_or(nil, nil), nil)
    luaunit.assertEquals(unit.option_string_or("Bilbo", nil), "Bilbo")
    luaunit.assertEquals(unit.option_string_or(nil, "Frodo"), "Frodo")
    luaunit.assertEquals(unit.option_string_or("Bilbo", "Frodo"), "Bilbo")

    luaunit.assertEquals(unit.option_vec_i32_or(nil, nil), nil)
    luaunit.assertEquals(unit.option_vec_i32_or({ 1, 2, 3 }, nil), { 1, 2, 3 })
    luaunit.assertEquals(unit.option_vec_i32_or(nil, { 1, 2, 3 }), { 1, 2, 3 })
    luaunit.assertEquals(unit.option_vec_i32_or({ 1, 2, 3 }, { 4, 5 }), { 1, 2, 3 })

    luaunit.assertEquals(unit.option_vec_string_or(nil, nil), nil)
    luaunit.assertEquals(unit.option_vec_string_or({ "Red", "Green", "Blue" }, nil), { "Red", "Green", "Blue" })
    luaunit.assertEquals(unit.option_vec_string_or(nil, { "Red", "Green", "Blue" }), { "Red", "Green", "Blue" })
    luaunit.assertEquals(unit.option_vec_string_or({ "Red", "Green" }, { "Blue" }), { "Red", "Green" })

    luaunit.assertEquals(unit.option_option_i32_or(nil, nil), nil)
    luaunit.assertEquals(unit.option_option_i32_or(42, nil), 42)
    luaunit.assertEquals(unit.option_option_i32_or(nil, 42), 42)
    luaunit.assertEquals(unit.option_option_i32_or(42, 43), 42)

    luaunit.assertEquals(unit.option_option_string_or(nil, nil), nil)
    luaunit.assertEquals(unit.option_option_string_or("Bilbo", nil), "Bilbo")
    luaunit.assertEquals(unit.option_option_string_or(nil, "Frodo"), "Frodo")
    luaunit.assertEquals(unit.option_option_string_or("Bilbo", "Frodo"), "Bilbo")
end

function M.testOptionStructOr()
    luaunit.assertEquals(unit.option_a_or(nil, nil), nil)
    luaunit.assertEquals(unit.option_a_or(unit.make_a("Bilbo Baggins", 42), nil).string, "Bilbo Baggins")
    luaunit.assertEquals(unit.option_a_or(nil, unit.make_a("Bilbo Baggins", 42)).string, "Bilbo Baggins")
    luaunit.assertEquals(unit.option_a_or(unit.make_a("Bilbo", 42), unit.make_a("Frodo", 42)).string, "Bilbo")

    luaunit.assertEquals(unit.option_option_a_or(nil, nil), nil)
    luaunit.assertEquals(unit.option_option_a_or(unit.make_a("Bilbo Baggins", 42), nil).string, "Bilbo Baggins")
    luaunit.assertEquals(unit.option_option_a_or(nil, unit.make_a("Bilbo Baggins", 42)).string, "Bilbo Baggins")
    luaunit.assertEquals(unit.option_option_a_or(unit.make_a("Bilbo", 42), unit.make_a("Frodo", 42)).string, "Bilbo")
end

function M.testOptionVecStructOr()
    luaunit.assertEquals(unit.option_vec_a_or(nil, nil), nil)
    local as = unit.option_vec_a_or({ unit.make_a("Bilbo", 42), unit.make_a("Baggins", 43) }, nil)
    luaunit.assertEquals(#as, 2)
    luaunit.assertEquals(as[1].string, "Bilbo")
    luaunit.assertEquals(as[1].integer, 42)
    luaunit.assertEquals(as[2].string, "Baggins")
    luaunit.assertEquals(as[2].integer, 43)
    local as = unit.option_vec_a_or(nil, { unit.make_a("Bilbo", 42), unit.make_a("Baggins", 43) })
    luaunit.assertEquals(#as, 2)
    luaunit.assertEquals(as[1].string, "Bilbo")
    luaunit.assertEquals(as[1].integer, 42)
    luaunit.assertEquals(as[2].string, "Baggins")
    luaunit.assertEquals(as[2].integer, 43)
    local as = unit.option_vec_a_or({ unit.make_a("Bilbo", 42), unit.make_a("Baggins", 43) }, {})
    luaunit.assertEquals(#as, 2)
    luaunit.assertEquals(as[1].string, "Bilbo")
    luaunit.assertEquals(as[1].integer, 42)
    luaunit.assertEquals(as[2].string, "Baggins")
    luaunit.assertEquals(as[2].integer, 43)
end

function M.testConcatenateVecStructs()
    local as = unit.concatenate_vec_a(
        {
            unit.make_a("Dead", 0xDEAD),
            unit.make_a("Beef", 0xBEEF)
        },
        {
            unit.make_a("Covfefe", 0xC0FEFE)
        })
    luaunit.assertEquals(#as, 3)
    luaunit.assertEquals(as[1].string, "Dead")
    luaunit.assertEquals(as[1].integer, 0xDEAD)
    luaunit.assertEquals(as[2].string, "Beef")
    luaunit.assertEquals(as[2].integer, 0xBEEF)
    luaunit.assertEquals(as[3].string, "Covfefe")
    luaunit.assertEquals(as[3].integer, 0xC0FEFE)
end

function M.testMakeStruct()
    local a = unit.make_a("Covfefe", 0xC0FEFE)
    luaunit.assertEquals(a.string, "Covfefe")
    luaunit.assertEquals(a.integer, 0xC0FEFE)

    local b = unit.make_b(nil, nil)
    luaunit.assertEquals(b.string, nil)
    luaunit.assertEquals(b.integer, nil)
    local b = unit.make_b("Covfefe", nil)
    luaunit.assertEquals(b.string, "Covfefe")
    luaunit.assertEquals(b.integer, nil)
    local b = unit.make_b(nil, 0xC0FEFE)
    luaunit.assertEquals(b.string, nil)
    luaunit.assertEquals(b.integer, 0xC0FEFE)
    local b = unit.make_b("Beef", 0xBEEF)
    luaunit.assertEquals(b.string, "Beef")
    luaunit.assertEquals(b.integer, 0xBEEF)

    local c = unit.make_c(nil, {})
    luaunit.assertEquals(c.a, nil)
    luaunit.assertEquals(c.b, {})
    local c = unit.make_c(unit.make_a("Bilbo Baggins", 111), {})
    luaunit.assertEquals(c.a.string, "Bilbo Baggins")
    luaunit.assertEquals(c.a.integer, 111)
    luaunit.assertEquals(c.b, {})
    local c = unit.make_c(unit.make_a("Bilbo Baggins", 111), { unit.make_b("Frodo Baggins", nil) })
    luaunit.assertEquals(c.a.string, "Bilbo Baggins")
    luaunit.assertEquals(c.a.integer, 111)
    luaunit.assertEquals(#c.b, 1)
    luaunit.assertEquals(c.b[1].string, "Frodo Baggins")
    luaunit.assertEquals(c.b[1].integer, nil)

    luaunit.assertEquals(unit.make_d({ 1, 2, 3, 4 }).integers, { 1, 2, 3, 4 })
    luaunit.assertEquals(unit.make_d({}).integers, {})

    local f = unit.make_f(nil, nil)
    luaunit.assertEquals(f.as_, nil)
    luaunit.assertEquals(f.strings, nil)
    local f = unit.make_f({}, nil)
    luaunit.assertEquals(f.as_, {})
    luaunit.assertEquals(f.strings, nil)
    local f = unit.make_f(nil, {})
    luaunit.assertEquals(f.as_, nil)
    luaunit.assertEquals(f.strings, {})
    local f = unit.make_f({ unit.make_a("Covfefe", 0xC0FEFE) }, nil)
    luaunit.assertEquals(#f.as_, 1)
    luaunit.assertEquals(f.as_[1].string, "Covfefe")
    luaunit.assertEquals(f.strings, nil)
    local f = unit.make_f(nil, { "Bilbo", "Baggins"})
    luaunit.assertEquals(f.as_, nil)
    luaunit.assertEquals(f.strings, { "Bilbo", "Baggins" })
end

function M.testConsumeStructs()
    luaunit.assertEquals(
        unit.describe(
            unit.make_a("A", 1),
            unit.make_b("B", 2),
            unit.make_c(unit.make_a("CA", 3), { unit.make_b("CB", 4) }),
            unit.make_d({ 5, 6, 7 }),
            unit.make_e({ 8, 9 }, { unit.make_d({ 10, 11 }) }),
            unit.make_f({ unit.make_a("FA1", 12), unit.make_a("FA2", 13) }, { "S1", "S2" })
        ),
        'A: A { string: "A", integer: 1 }, '
        ..'B: B { string: Some("B"), integer: Some(2) }, '
        ..'C: C { a: Some(A { string: "CA", integer: 3 }), b: [B { string: Some("CB"), integer: Some(4) }] }, '
        ..'D: D { integers: [5, 6, 7] }, '
        ..'E: E { integers: Some([8, 9]), ds: [D { integers: [10, 11] }] }, '
        ..'F: F { as_: Some([A { string: "FA1", integer: 12 }, A { string: "FA2", integer: 13 }]), strings: Some(["S1", "S2"]) }'
    )
end

function M.testPanic()
    local status, msg = pcall(unit.i_like_to_panic)
    luaunit.assertFalse(status)
    luaunit.assertNotNil(msg)
end

function M.testU8s()
    local string = "Hello World!"
    luaunit.assertEquals(unit.u8_slice_to_string(string), string)
    luaunit.assertEquals(unit.u8_vec_to_string(string), string)
end

function M.testInvalidUtf8()
    local string = "\0\x9f"
    local status, _ = pcall(unit.u8_slice_to_string, string)
    luaunit.assertFalse(status)
    local status, string = pcall(unit.u8_vec_to_string, string)
    luaunit.assertFalse(status)
end

function M.testStringWithByteZeros()
    local status, _ = pcall(unit.string_with_byte_zeros)
    luaunit.assertFalse(status)
end

function M.testBooleans()
    local g1 = unit.make_g(true, nil, {})
    local g2 = unit.make_g(false, true, { true })
    local g3 = unit.make_g(true, false, { true, false })

    luaunit.assertTrue(g1.b)
    luaunit.assertNil(g1.option_b)
    luaunit.assertEquals(#g1.vec_b, 0)

    luaunit.assertFalse(g2.b)
    luaunit.assertTrue(g2.option_b)
    luaunit.assertEquals(#g2.vec_b, 1)
    luaunit.assertTrue(g2.vec_b[1])

    luaunit.assertTrue(g3.b)
    luaunit.assertFalse(g3.option_b)
    luaunit.assertEquals(#g3.vec_b, 2)
    luaunit.assertTrue(g3.vec_b[1])
    luaunit.assertFalse(g3.vec_b[2])
end

function M.testBlob()
    local message = "Message"
    local blob = unit.blob_string(message)
    luaunit.assertEquals(unit.use_blob_string(blob), message)
end

function M.testResult()
    local nil_nil_a = unit.maybe_make_a(nil, nil)
    luaunit.assertNil(nil_nil_a.ok)
    luaunit.assertEquals(nil_nil_a.err, "Neither")

    local some_nil_a = unit.maybe_make_a("Bilbo Baggins", nil)
    luaunit.assertNil(some_nil_a.ok)
    luaunit.assertEquals(some_nil_a.err, "No integer")

    local nil_some_a = unit.maybe_make_a(nil, 42)
    luaunit.assertNil(nil_some_a.ok)
    luaunit.assertEquals(nil_some_a.err, "No string")

    local some_some_a = unit.maybe_make_a("Bilbo Baggins", 111)
    luaunit.assertNil(some_some_a.err)
    luaunit.assertEquals(some_some_a.ok.string, "Bilbo Baggins")
    luaunit.assertEquals(some_some_a.ok.integer, 111)

    local ok_a = unit.make_b_or_a("Bilbo Baggins", 111)
    luaunit.assertNil(ok_a.err)
    luaunit.assertEquals(ok_a.ok.string, "Bilbo Baggins")
    luaunit.assertEquals(ok_a.ok.integer, 111)

    local err_b = unit.make_b_or_a("Bilbo Baggins", nil)
    luaunit.assertNil(err_b.ok)
    luaunit.assertEquals(err_b.err.string, "Bilbo Baggins")
    luaunit.assertNil(err_b.err.integer)

    local err_b = unit.make_b_or_a(nil, 42)
    luaunit.assertNil(err_b.ok)
    luaunit.assertNil(err_b.err.string)
    luaunit.assertEquals(err_b.err.integer, 42)

    local ok_none = unit.ok_none()
    luaunit.assertNil(ok_none.ok)
    luaunit.assertNil(ok_none.err)

    local err_none = unit.ok_none()
    luaunit.assertNil(err_none.ok)
    luaunit.assertNil(err_none.err)
end

return M
