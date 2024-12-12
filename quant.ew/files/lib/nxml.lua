local ffi = nil
if require then
    pcall(function()
        ffi = require("ffi")
    end)
end

local str_sub
local str_index
local str_normalize

if ffi then
    str_normalize = function(str)
        return ffi.cast("const char*", str)
    end

    str_sub = function(ptr, start_idx, len)
        return ffi.string(ptr + start_idx, len)
    end

    str_index = function(ptr, idx)
        return ptr[idx]
    end
else
    str_normalize = function(str) return str end

    str_sub = function(str, start_idx, len)
        return str:sub(start_idx + 1, start_idx + len)
    end

    str_index = function(str, idx)
        return string.byte(str:sub(idx + 1, idx + 1))
    end
end

--[[
 * The following is a Lua port of the NXML parser:
 * https://github.com/xwitchproject/nxml
 *
 * The NXML Parser is heavily based on code from poro
 * https://github.com/gummikana/poro
 * 
 * The poro project is licensed under the Zlib license:
 * 
 * --------------------------------------------------------------------------
 * Copyright (c) 2010-2019 Petri Purho, Dennis Belfrage
 * Contributors: Martin Jonasson, Olli Harjola
 * This software is provided 'as-is', without any express or implied
 * warranty.  In no event will the authors be held liable for any damages
 * arising from the use of this software.
 * Permission is granted to anyone to use this software for any purpose,
 * including commercial applications, and to alter it and redistribute it
 * freely, subject to the following restrictions:
 * 1. The origin of this software must not be misrepresented; you must not
 *    claim that you wrote the original software. If you use this software
 *    in a product, an acknowledgment in the product documentation would be
 *    appreciated but is not required.
 * 2. Altered source versions must be plainly marked as such, and must not be
 *    misrepresented as being the original software.
 * 3. This notice may not be removed or altered from any source distribution.
 * --------------------------------------------------------------------------
]]

local nxml = {}

local TOKENIZER_FUNCS = {}
local TOKENIZER_MT = {
    __index = TOKENIZER_FUNCS,
    __tostring = function(self) return "natif.nxml.tokenizer" end
}

local function new_tokenizer(cstring, len)
    return setmetatable({
        data = cstring,
        cur_idx = 0,
        cur_row = 1,
        cur_col = 1,
        prev_row = 1,
        prev_col = 1,
        len = len
    }, TOKENIZER_MT)
end

local ws = {
    [string.byte(" ")] = true,
    [string.byte("\t")] = true,
    [string.byte("\n")] = true,
    [string.byte("\r")] = true
}

function TOKENIZER_FUNCS:is_whitespace(char)
    local n = tonumber(char)
    return ws[n] or false
end

local punct = {
    [string.byte("<")] = true,
    [string.byte(">")] = true,
    [string.byte("=")] = true,
    [string.byte("/")] = true,
}

function TOKENIZER_FUNCS:is_whitespace_or_punctuation(char)
    local n = tonumber(char)
    return self:is_whitespace(n) or punct[n] or false
end

function TOKENIZER_FUNCS:move(n)
    n = n or 1
    local prev_idx = self.cur_idx
    self.cur_idx = self.cur_idx + n
    if self.cur_idx >= self.len then
        self.cur_idx = self.len
        return
    end
    for i = prev_idx, self.cur_idx - 1 do
        if str_index(self.data, i) == string.byte("\n") then
            self.cur_row = self.cur_row + 1
            self.cur_col = 1
        else
            self.cur_col = self.cur_col + 1
        end
    end
end

function TOKENIZER_FUNCS:peek(n)
    n = n or 1
    local idx = self.cur_idx + n
    if idx >= self.len then return 0 end

    return str_index(self.data, idx)
end

function TOKENIZER_FUNCS:match_string(str)
    local len = #str
    str = str_normalize(str)
    
    for i = 0, len - 1 do
        if self:peek(i) ~= str_index(str, i) then return false end
    end
    return true
end

function TOKENIZER_FUNCS:eof()
    return self.cur_idx >= self.len
end

function TOKENIZER_FUNCS:cur_char()
    if self:eof() then return 0 end
    return tonumber(str_index(self.data, self.cur_idx))
end

function TOKENIZER_FUNCS:skip_whitespace()
    while not self:eof() do
        if self:is_whitespace(self:cur_char()) then
            self:move()
        elseif self:match_string("<!--") then
            self:move(4)
            while not self:eof() and not self:match_string("-->") do
                self:move()
            end

            if self:match_string("-->") then
                self:move(3)
            end
        elseif self:cur_char() == string.byte("<") and self:peek(1) == string.byte("!") then
            self:move(2)
            while not self:eof() and self:cur_char() ~= string.byte(">") do
                self:move()
            end
            if self:cur_char() == string.byte(">") then
                self:move()
            end

        elseif self:match_string("<?") then
            self:move(2)
            while not self:eof() and not self:match_string("?>") do
                self:move()
            end
            if self:match_string("?>") then
                self:move(2)
            end
        else
            break
        end
    end
end

function TOKENIZER_FUNCS:read_quoted_string()
    local start_idx = self.cur_idx
    local len = 0

    while not self:eof() and self:cur_char() ~= string.byte("\"") do
        len = len + 1
        self:move()
    end

    self:move() -- skip "
    return str_sub(self.data, start_idx, len)
end

function TOKENIZER_FUNCS:read_unquoted_string()
    local start_idx = self.cur_idx - 1 -- first char is move()d
    local len = 1

    while not self:eof() and not self:is_whitespace_or_punctuation(self:cur_char()) do
        len = len + 1
        self:move()
    end

    return str_sub(self.data, start_idx, len)
end

local C_NULL = 0
local C_LT = string.byte("<")
local C_GT = string.byte(">")
local C_SLASH = string.byte("/")
local C_EQ = string.byte("=")
local C_QUOTE = string.byte("\"")

function TOKENIZER_FUNCS:next_token()
    self:skip_whitespace()

    self.prev_row = self.cur_row
    self.prev_col = self.cur_col

    if self:eof() then return nil end

    local c = self:cur_char()
    self:move()

    if c == C_NULL then return nil
    elseif c == C_LT then return { type = "<" }
    elseif c == C_GT then return { type = ">" }
    elseif c == C_SLASH then return { type = "/" }
    elseif c == C_EQ then return { type = "=" }
    elseif c == C_QUOTE then return { type = "string", value = self:read_quoted_string() }
    else return { type = "string", value = self:read_unquoted_string() }
    end
end

local PARSER_FUNCS = {}
local PARSER_MT = {
    __index = PARSER_FUNCS,
    __tostring = function(self) return "natif.nxml.parser" end
}

local function new_parser(tokenizer, error_reporter)
    return setmetatable({
        tok = tokenizer,
        errors = {},
        error_reporter = error_reporter or function(type, msg) print("parser error: [" .. type .. "] " .. msg) end
    }, PARSER_MT)
end

local XML_ELEMENT_FUNCS = {}
local XML_ELEMENT_MT = {
    __index = XML_ELEMENT_FUNCS,
    __tostring = function(self)
        return nxml.tostring(self)
    end,
}

function PARSER_FUNCS:report_error(type, msg)
    self.error_reporter(type, msg)
    table.insert(self.errors, { type = type, msg = msg, row = self.tok.prev_row, col = self.tok.prev_col })
end


function PARSER_FUNCS:parse_attr(attr_table, name)
    local tok = self.tok:next_token()
    if tok.type == "=" then
        tok = self.tok:next_token()

        if tok.type == "string" then
            attr_table[name] = tok.value
        else
            self:report_error("missing_attribute_value", string.format("parsing attribute '%s' - expected a string after =, but did not find one"), name)
        end
    else
        self:report_error("missing_equals_sign", string.format("parsing attribute '%s' - did not find equals sign after attribute name", name))
    end
end

function PARSER_FUNCS:parse_element(skip_opening_tag)
    local tok
    if not skip_opening_tag then
        tok = self.tok:next_token()
        if tok.type ~= "<" then
            self:report_error("missing_tag_open", "couldn't find a '<' to start parsing with")
        end
    end

    tok = self.tok:next_token()
    if tok.type ~= "string" then
        self:report_error("missing_element_name", "expected an element name after '<'")
    end

    local elem_name = tok.value
    local elem = nxml.new_element(elem_name)
    local content_idx = 0

    local self_closing = false

    while true do
        tok = self.tok:next_token()

        if tok == nil then
            return elem
        elseif tok.type == "/" then
            if self.tok:cur_char() == C_GT then
                self.tok:move()
                self_closing = true
            end
            break
        elseif tok.type == ">" then
            break
        elseif tok.type == "string" then
            self:parse_attr(elem.attr, tok.value)
        end
    end

    if self_closing then return elem end

    while true do
        tok = self.tok:next_token()

        if tok == nil then
            return elem
        elseif tok.type == "<" then
            if self.tok:cur_char() == C_SLASH then
                self.tok:move()

                local end_name = self.tok:next_token()
                if end_name.type == "string" and end_name.value == elem_name then
                    local close_greater = self.tok:next_token()

                    if close_greater.type == ">" then
                        return elem
                    else
                        self:report_error("missing_element_close", string.format("no closing '>' found for element '%s'", elem_name))
                    end
                else
                    self:report_error("mismatched_closing_tag", string.format("closing element is in wrong order - expected '</%s>', but instead got '%s'", elem_name, tostring(end_name.value)))
                end
                return elem
            else
                local child = self:parse_element(elem, true)
                table.insert(elem.children, child)
            end
        else
            if not elem.content then
                elem.content = {}
            end
            
            content_idx = content_idx + 1
            elem.content[content_idx] = tok.value or tok.type
        end
    end
end

function PARSER_FUNCS:parse_elements()
    local tok = self.tok:next_token()
    local elems = {}
    local elems_i = 1

    while tok and tok.type == "<" do
        elems[elems_i] = self:parse_element(true)
        elems_i = elems_i + 1

        tok = self.tok:next_token()
    end

    return elems
end

local function is_punctuation(str)
    return str == "/" or str == "<" or str == ">" or str == "="
end

function XML_ELEMENT_FUNCS:text()
    local content_count = #self.content

    if self.content == nil or content_count == 0 then
        return ""
    end

    local text = self.content[1]
    for i = 2, content_count do
        local elem = self.content[i]
        local prev = self.content[i - 1]

        if is_punctuation(elem) or is_punctuation(prev) then
            text = text .. elem
        else
            text = text .. " " .. elem
        end
    end

    return text
end

function XML_ELEMENT_FUNCS:add_child(child)
    self.children[#self.children + 1] = child
end

function XML_ELEMENT_FUNCS:add_children(children)
    local children_i = #self.children + 1
    for i = 1, #children do
        self.children[children_i] = children[i]
        children_i = children_i + 1
    end
end

function XML_ELEMENT_FUNCS:remove_child(child)
    for i = 1, #self.children do
        if self.children[i] == child then
            table.remove(self.children, i)
            break
        end
    end
end

function XML_ELEMENT_FUNCS:remove_child_at(index)
    table.remove(self.children, index)
end

function XML_ELEMENT_FUNCS:clear_children()
    self.children = {}
end

function XML_ELEMENT_FUNCS:clear_attrs()
    self.attr = {}
end

function XML_ELEMENT_FUNCS:first_of(element_name)
    local i = 0
    local n = #self.children

    while i < n do
        i = i + 1
        local c = self.children[i]

        if c.name == element_name then return c end
    end

    return nil
end

function XML_ELEMENT_FUNCS:each_of(element_name)
    local i = 1
    local n = #self.children

    return function()
        while i <= n and self.children[i].name ~= element_name do
            i = i + 1
        end
        i = i + 1
        return self.children[i - 1]
    end
end

function XML_ELEMENT_FUNCS:all_of(element_name)
    local table = {}
    local i = 1
    for elem in self:each_of(element_name) do
        table[i] = elem
        i = i + 1
    end
    return table
end

function XML_ELEMENT_FUNCS:each_child()
    local i = 0
    local n = #self.children

    return function()
        while i <= n do
            i = i + 1
            return self.children[i]
        end
    end
end

function nxml.parse(data)
    local data_len = #data
    local tok = new_tokenizer(str_normalize(data), data_len)
    local parser = new_parser(tok)
    
    local elem = parser:parse_element(false)
    
    if not elem or (elem.errors and #elem.errors > 0) then
        error("parser encountered errors")
    end

    return elem
end

function nxml.parse_many(data)
    local data_len = #data
    local tok = new_tokenizer(str_normalize(data), data_len)
    local parser = new_parser(tok)
    
    local elems = parser:parse_elements(false)
    
    for i = 1, #elems do
        local elem = elems[i]

        if elem.errors and #elem.errors > 0 then
            error("parser encountered errors")
        end
    end

    return elems
end

function nxml.new_element(name, attrs)
    return setmetatable({
        name = name,
        attr = attrs or {},
        children = {},
        content = nil
    }, XML_ELEMENT_MT)
end

local function attr_value_to_str(value)
    local t = type(value)
    if t == "string" then return value end
    if t == "boolean" then return value and "1" or "0" end

    return tostring(value)
end

function nxml.tostring(elem, packed, indent_char, cur_indent)
    indent_char = indent_char or "\t"
    cur_indent = cur_indent or ""
    local s = "<" .. elem.name
    local self_closing = #elem.children == 0 and (not elem.content or #elem.content == 0)

    for k, v in pairs(elem.attr) do
        s = s .. " " .. k .. "=\"" .. attr_value_to_str(v) .. "\""
    end

    if self_closing then
        s = s .. " />"
        return s
    end

    s = s .. ">"

    local deeper_indent = cur_indent .. indent_char

    if elem.content and #elem.content ~= 0 then
        if not packed then s = s .. "\n" .. deeper_indent end
        s = s .. elem:text()
    end

    if not packed then s = s .. "\n" end

    for i, v in ipairs(elem.children) do
        if not packed then s = s .. deeper_indent end
        s = s .. nxml.tostring(v, packed, indent_char, deeper_indent)
        if not packed then s = s .. "\n" end
    end

    s = s .. cur_indent .. "</" .. elem.name .. ">"

    return s
end

return nxml
