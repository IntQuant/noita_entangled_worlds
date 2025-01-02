---@alias int integer
---@alias bool boolean
---@alias str string
---@alias token_type "string" | "<" | ">" | "/" | "="
---@alias error_type "missing_attribute_value" | "missing_element_close" | "missing_equals_sign" | "missing_element_name" | "missing_tag_open" | "mismatched_closing_tag" | "missing_token" | "missing_element" | "duplicate_attribute"
---@alias error_fn fun(type: error_type, msg: str)

---@class (exact) token
---@field value string?
---@field type token_type

---@class (exact) error
---@field type error_type
---@field msg str
---@field row int
---@field col int

---@class (exact) tokenizer: tokenizer_funcs
---@field data str
---@field cur_idx int
---@field cur_row int
---@field cur_col int
---@field prev_row int
---@field prev_col int
---@field len int

---@class (exact) parser: parser_funcs
---@field tok tokenizer
---@field errors error[]
---@field error_reporter error_fn

---@class (exact) element: element_funcs
---@field content str[]?
---@field children element[]
---@field attr table<string, string>
---@field name str
---@field errors error[]

---@param str str
---@param start_idx int
---@param len int
---@return str
local function str_sub(str, start_idx, len)
    return str:sub(start_idx + 1, start_idx + len)
end

---@param str str
---@param idx int
---@return integer
local function str_index(str, idx)
    return string.byte(str:sub(idx + 1, idx + 1))
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

---@class nxml
local nxml = {}
---@type fun(type: error_type, msg: str)?
nxml.error_handler = nil

---@class tokenizer_funcs
local TOKENIZER_FUNCS = {}
local TOKENIZER_MT = {
    __index = TOKENIZER_FUNCS,
    __tostring = function(_)
        return "natif.nxml.tokenizer"
    end,
}

---@param cstring str
---@return tokenizer
local function new_tokenizer(cstring)
    ---@type tokenizer
    local tokenizer = {
        data = cstring,
        cur_idx = 0,
        cur_row = 1,
        cur_col = 1,
        prev_row = 1,
        prev_col = 1,
        len = #cstring,
    }
    -- idk why luals doesn't like this
    ---@diagnostic disable-next-line: return-type-mismatch
    return setmetatable(tokenizer, TOKENIZER_MT)
end

---@type table<int, bool>
local ws = {
    [string.byte(" ")] = true,
    [string.byte("\t")] = true,
    [string.byte("\n")] = true,
    [string.byte("\r")] = true,
}

---@param char int
---@return bool
function TOKENIZER_FUNCS:is_whitespace(char)
    local n = tonumber(char)
    return ws[n] or false
end

---@type table<int, bool>
local punct = {
    [string.byte("<")] = true,
    [string.byte(">")] = true,
    [string.byte("=")] = true,
    [string.byte("/")] = true,
}

---@param char int
---@return bool
function TOKENIZER_FUNCS:is_whitespace_or_punctuation(char)
    local n = tonumber(char)
    -- We can disable here because is_whitespace(!int) -> false
    ---@diagnostic disable-next-line: param-type-mismatch
    return self:is_whitespace(n) or punct[n] or false
end

---@param n int? 1
function TOKENIZER_FUNCS:move(n)
    ---@cast self tokenizer
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

---@param n int? 1
---@return int
function TOKENIZER_FUNCS:peek(n)
    ---@cast self tokenizer
    n = n or 1
    local idx = self.cur_idx + n
    if idx >= self.len then
        return 0
    end

    return str_index(self.data, idx)
end

---@param str str
---@return boolean
function TOKENIZER_FUNCS:match_string(str)
    local len = #str

    for i = 0, len - 1 do
        if self:peek(i) ~= str_index(str, i) then
            return false
        end
    end
    return true
end

---@return bool
function TOKENIZER_FUNCS:eof()
    ---@cast self tokenizer
    return self.cur_idx >= self.len
end

---@return int
function TOKENIZER_FUNCS:cur_char()
    ---@cast self tokenizer
    if self:eof() then
        return 0
    end
    return str_index(self.data, self.cur_idx)
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

---@return str
function TOKENIZER_FUNCS:read_quoted_string()
    ---@cast self tokenizer
    local start_idx = self.cur_idx
    local len = 0

    while not self:eof() and self:cur_char() ~= string.byte('"') do
        len = len + 1
        self:move()
    end

    self:move() -- skip "
    return str_sub(self.data, start_idx, len)
end

---@return str
function TOKENIZER_FUNCS:read_unquoted_string()
    ---@cast self tokenizer
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
local C_QUOTE = string.byte('"')

---@return token?
function TOKENIZER_FUNCS:next_token()
    self:skip_whitespace()

    self.prev_row = self.cur_row
    self.prev_col = self.cur_col

    if self:eof() then
        return nil
    end

    local c = self:cur_char()
    self:move()

    if c == C_NULL then
        return nil
    elseif c == C_LT then
        ---@type token
        local v = { type = "<" }
        return v
    elseif c == C_GT then
        ---@type token
        local v = { type = ">" }
        return v
    elseif c == C_SLASH then
        ---@type token
        local v = { type = "/" }
        return v
    elseif c == C_EQ then
        ---@type token
        local v = { type = "=" }
        return v
    elseif c == C_QUOTE then
        ---@type token
        local v = { type = "string", value = self:read_quoted_string() }
        return v
    else
        ---@type token
        local v = { type = "string", value = self:read_unquoted_string() }
        return v
    end
end

---@class parser_funcs
local PARSER_FUNCS = {}
local PARSER_MT = {
    __index = PARSER_FUNCS,
    __tostring = function(_)
        return "natif.nxml.parser"
    end,
}

---@param tokenizer tokenizer
---@param error_reporter fun(type: error_type, msg: str)?
---@return parser | parser_funcs parser
local function new_parser(tokenizer, error_reporter)
    ---@type parser
    local parser = {
        tok = tokenizer,
        errors = {},
        error_reporter = error_reporter or function(type, msg)
            print("parser error: [" .. type .. "] " .. msg)
        end,
    }
    -- why does luals not care about here?
    return setmetatable(parser, PARSER_MT)
end

---@class element_funcs
local XML_ELEMENT_FUNCS = {}
local XML_ELEMENT_MT = {
    __index = XML_ELEMENT_FUNCS,
    __tostring = function(self)
        return nxml.tostring(self, false)
    end,
}

---@param type error_type
---@param msg str
function PARSER_FUNCS:report_error(type, msg)
    ---@cast self parser
    self.error_reporter(type, msg)
    ---@type error
    local error = { type = type, msg = msg, row = self.tok.prev_row, col = self.tok.prev_col }
    table.insert(self.errors, error)
end

---@param attr_table table<str, str>
---@param name str
function PARSER_FUNCS:parse_attr(attr_table, name)
    ---@cast self parser
    local tok = self.tok:next_token()
    if not tok then
        self:report_error("missing_token", string.format("parsing attribute '%s' - did not find a token", name))
        return
    end
    if tok.type == "=" then
        tok = self.tok:next_token()

        if not tok then
            self:report_error("missing_token", string.format("parsing attribute '%s' - did not find a token", name))
            return
        end

        if tok.type == "string" then
            if attr_table[name] ~= nil then
                self:report_error(
                    "duplicate_attribute",
                    string.format("parsing attribute '%s' - attribute already exists", name)
                )
                return
            end
            attr_table[name] = tok.value
        else
            self:report_error(
                "missing_attribute_value",
                string.format("parsing attribute '%s' - expected a string after =, but did not find one", name)
            )
        end
    else
        self:report_error(
            "missing_equals_sign",
            string.format("parsing attribute '%s' - did not find equals sign after attribute name", name)
        )
    end
end

---@param skip_opening_tag bool
---@return element?
function PARSER_FUNCS:parse_element(skip_opening_tag)
    ---@cast self parser
    local tok
    if not skip_opening_tag then
        tok = self.tok:next_token()
        if not tok then
            self:report_error("missing_token", "parsing element - did not find a token")
            return
        end
        if tok.type ~= "<" then
            self:report_error("missing_tag_open", "couldn't find a '<' to start parsing with")
        end
    end

    tok = self.tok:next_token()
    if not tok then
        self:report_error("missing_token", "parsing element - did not find a token")
        return
    end
    if tok.type ~= "string" then
        self:report_error("missing_element_name", "expected an element name after '<'")
    end

    local elem_name = tok.value
    if not elem_name then
        self:report_error("missing_attribute_value", "parse element element missing name")
        return
    end
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

    if self_closing then
        return elem
    end

    while true do
        tok = self.tok:next_token()

        if tok == nil then
            return elem
        elseif tok.type == "<" then
            if self.tok:cur_char() == C_SLASH then
                self.tok:move()

                local end_name = self.tok:next_token()
                if not end_name then
                    self:report_error(
                        "missing_token",
                        string.format("parsing element '%s' - did not find a token", elem_name)
                    )
                    return
                end
                if end_name.type == "string" and end_name.value == elem_name then
                    local close_greater = self.tok:next_token()
                    if not close_greater then
                        self:report_error(
                            "missing_token",
                            string.format("parsing element '%s' - did not find a token", elem_name)
                        )
                        return
                    end

                    if close_greater.type == ">" then
                        return elem
                    else
                        self:report_error(
                            "missing_element_close",
                            string.format("no closing '>' found for element '%s'", elem_name)
                        )
                    end
                else
                    self:report_error(
                        "mismatched_closing_tag",
                        string.format(
                            "closing element is in wrong order - expected '</%s>', but instead got '%s'",
                            elem_name,
                            tostring(end_name.value)
                        )
                    )
                end
                return elem
            else
                local child = self:parse_element(true)
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

---@return element[]
function PARSER_FUNCS:parse_elements()
    ---@cast self parser
    local tok = self.tok:next_token()
    ---@type element[]
    local elems = {}
    local elems_i = 1

    while tok and tok.type == "<" do
        local next_element = self:parse_element(true)
        if not next_element then
            self:report_error("missing_element", "parse_element returned nil while parsing elements")
            return elems
        end
        elems[elems_i] = next_element
        elems_i = elems_i + 1

        tok = self.tok:next_token()
    end

    return elems
end

---@param str str
---@return bool
local function is_punctuation(str)
    return str == "/" or str == "<" or str == ">" or str == "="
end

---Copies attributes from `source` to `dest` if dest doesn't have a value
---@param dest element
---@param source element
local function merge_element(dest, source)
    for attr_name, attr_value in pairs(source.attr) do
        if dest:get(attr_name) == nil then
            dest:set(attr_name, attr_value)
        end
    end
end

---Merge the content of the base file into the child tree
---@param root element
---@param base_element element
---@param base_file element
local function merge_xml(root, base_element, base_file)
    local index = 1
    local counts = {}
    for elem in base_file:each_child() do
        if not counts[elem.name] then
            counts[elem.name] = 0
        end
        counts[elem.name] = counts[elem.name] + 1
        local modifications = base_element:nth_of(elem.name, counts[elem.name])
        if modifications then
            merge_element(modifications, elem)
            --[[if #elem.children > 0 then
				merge_xml(root, base, elem)
			end]]
        else
            table.insert(base_element.children, index, elem)
            index = index + 1
        end
    end

    for attr_name, attr_value in pairs(base_file.attr) do
        if not root:get(attr_name) then
            root:set(attr_name, attr_value)
        elseif attr_name == "tags" then
            local tags = root:get("tags") .. "," .. attr_value
            local tag_list = {}
            local tag_table = {}
            for tag in tags:gmatch("([^,]+)") do
                if tag ~= "" and not tag_table[tag] then
                    table.insert(tag_list, tag)
                    tag_table[tag] = 1
                end
            end
            tags = table.concat(tag_list, ",")
            root:set(attr_name, tags)
        end
    end

    --[[
	TODO:
	local to_remove = {}
	for idx, elem in ipairs(parent.children) do
		if elem.attr._remove_from_base == "1" then
			table.insert(to_remove, 1, idx)
		end
	end
	for _, idx in ipairs(to_remove) do
		table.remove(parent.children, idx)
	end
]]
end

---Expands the Base files for an entity xml
---Returns `self` for chaining purposes.
---**WARN: This is not 100% identical to Nollas implementation, _remove_from_base does not work**
---
---@param read (fun(path: str): str)? `ModTextFileGetContent`
---@param exists (fun(path: str): bool)? `ModDoesFileExist`
---@return element self
function XML_ELEMENT_FUNCS:expand_base(read, exists)
    ---@cast self element
    if self.name ~= "Entity" then
        return self
    end
    ---@cast self element
    -- thanks Kaedenn for writing this!
    read = read or ModTextFileGetContent
    exists = exists or ModDoesFileExist
    local base_tag
    while true do
        base_tag = self:first_of("Base")
        if not base_tag then
            break
        end
        local file = base_tag:get("file")
        if file and exists(file) then
            local root_xml = nxml.parse_file(file, read)

            root_xml:expand_base(read, exists)

            merge_xml(self, base_tag, root_xml)
            self:lift_child(base_tag)
        else
            self:remove_child(base_tag)
        end
    end
    for elem in self:each_child() do
        elem:expand_base(read, exists)
    end
    return self
end

---Returns `self` for chaining purposes.
---@param defaults table<string, table<string, any>>
---@return element
function XML_ELEMENT_FUNCS:apply_defaults(defaults)
    ---@cast self element
    local apply = defaults[self.name]
    for child in self:each_child() do
        child:apply_defaults(defaults)
    end
    if not apply then
        return self
    end
    for k, v in pairs(apply) do
        if self:get(k) == nil then
            self:set(k, v)
        end
    end
    return self
end

---Returns the content inside an element.
---Example:
---```xml
---<Hi>Content</Hi>
---```
---Here `:text()` is "Content"
---@return str
function XML_ELEMENT_FUNCS:text()
    ---@cast self element
    if self.content == nil then
        return ""
    end
    local content_count = #self.content
    if content_count == 0 then
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

---Returns `self` for chaining purposes.
---@param child element
---@return element self
function XML_ELEMENT_FUNCS:add_child(child)
    ---@cast self element
    self.children[#self.children + 1] = child
    return self
end

---Returns `self` for chaining purposes.
---@param children element[]
---@return element self
function XML_ELEMENT_FUNCS:add_children(children)
    ---@cast self element
    for _, child in ipairs(children) do
        self:add_child(child)
    end
    return self
end

---Creates a new element and adds it as a child to this element.
---Convenience function that combines xml:add_child with nxml.new_element.
---
---Example usage:
---```
--- elem:create_child("LifetimeComponent", { lifetime = 30 })
---```
---@param name str
---@param attrs table<str, any>? description of child element
---@param children element[]? child elements
---@return element new the element that was created
function XML_ELEMENT_FUNCS:create_child(name, attrs, children)
    local elem = nxml.new_element(name, attrs, children)
    self:add_child(elem)
    return elem
end

---Creates several new elements and inserts them as children to this element.
---Convenience function that combines xml:add_children with nxml.new_element.
---
---Example usage:
---```
---	elem:create_children(
---		{ AbilityComponent = {
---			ui_name = "$item_jar_with_mat"
---		}},
---		{ DamageModelComponent = {
---			hp = 2
---		}}
---	)
---```
---@param ... table<str, table<str,any>> descriptions of child elements
---@return element self for chaining purposes
function XML_ELEMENT_FUNCS:create_children(...)
    local elems = {}
    for _, elem_desc in ipairs({ ... }) do
        for name, attrs in pairs(elem_desc) do
            table.insert(elems, nxml.new_element(name, attrs))
        end
    end
    return self:add_children(elems)
end

---Removes the given child, note that this is exact equality not structural equality so copies will not be considered equal. Returns `self` for chaining purposes.
---@param child element
---@return element self
function XML_ELEMENT_FUNCS:remove_child(child)
    ---@cast self element
    for i = 1, #self.children do
        if self.children[i] == child then
            table.remove(self.children, i)
            break
        end
    end
    return self
end

---Removes the given child, but adds its children to this element. Returns `self` for chaining purposes
---@param child element
---@return element
function XML_ELEMENT_FUNCS:lift_child(child)
    ---@cast self element
    for k, v in ipairs(self.children) do
        if v == child then
            local dst_index = k + 1
            for elem in child:each_child() do
                table.insert(self.children, dst_index, elem)
                dst_index = dst_index + 1
            end
            table.remove(self.children, k)
            break
        end
    end
    return self
end

---Returns `self` for chaining purposes
---@param index int
---@return element
function XML_ELEMENT_FUNCS:remove_child_at(index)
    ---@cast self element
    table.remove(self.children, index)
    return self
end

---Returns `self` for chaining purposes
---@return element
function XML_ELEMENT_FUNCS:clear_children()
    ---@cast self element
    self.children = {}
    return self
end

---Returns `self` for chaining purposes
---@return element
function XML_ELEMENT_FUNCS:clear_attrs()
    ---@cast self element
    self.attr = {}
    return self
end

---Returns the first child element with the given name and its index.
---@param element_name str
---@return element?, int?
function XML_ELEMENT_FUNCS:first_of(element_name)
    ---@cast self element
    for k, v in ipairs(self.children) do
        if v.name == element_name then
            return v, k
        end
    end
end

---Returns the nth child element with the given name and its index.
---@param element_name str
---@param n int
---@return element?, int?
function XML_ELEMENT_FUNCS:nth_of(element_name, n)
    ---@cast self element
    for k, v in ipairs(self.children) do
        if v.name ~= element_name then
            goto continue
        end
        n = n - 1
        if n == 0 then
            return v, k
        end
        ::continue::
    end
end

---Iterate over each child with the given name, effectively a filter.
---Note that this function will behave strangely if you mutate children while iterating.
---Use like:
---```lua
---for dmc in entity:each_of("DamageModelComponent") do
---	dmc:set("hp", 5)
---end
---```
---@param element_name str
---@return fun(): element?
function XML_ELEMENT_FUNCS:each_of(element_name)
    ---@cast self element
    local i = 1
    local n = #self.children

    return function()
        while i <= n do
            local child = self.children[i]
            i = i + 1
            if child.name == element_name then
                return child
            end
        end
    end
end

---Collects all children with the given name into a table.
---@param element_name str
---@return element[]
function XML_ELEMENT_FUNCS:all_of(element_name)
    ---@cast self element
    local table = {}
    local i = 1
    for elem in self:each_of(element_name) do
        table[i] = elem
        i = i + 1
    end
    return table
end

---Iterate over each child of the xml element, use like:
---```lua
---for child in elem:each_child() do
---	print(child.name)
---end
---```
---@return fun(): element?
function XML_ELEMENT_FUNCS:each_child()
    ---@cast self element
    local i = 0
    local n = #self.children

    return function()
        while i < n do
            i = i + 1
            return self.children[i]
        end
    end
end

---@param value str | bool
---@return str
local function attr_value_to_str(value)
    local t = type(value)
    if t == "string" then
        return value
    end
    if t == "boolean" then
        return value and "1" or "0"
    end

    return tostring(value)
end

---Gets the given attribute, note get's value is probably stringified and not the true value.
---@param attr str
---@return str?
function XML_ELEMENT_FUNCS:get(attr)
    ---@cast self element
    return self.attr[attr]
end

---Sets the given attribute, make sure your type can be stringified. Returns `self` for chaining purposes.
---@param attr str
---@param value any
---@return element
function XML_ELEMENT_FUNCS:set(attr, value)
    ---@cast self element
    self.attr[attr] = attr_value_to_str(value)
    return self
end

---@return element
function XML_ELEMENT_FUNCS:clone()
    ---@cast self element
    local children = {}
    for e in self:each_child() do
        table.insert(children, e:clone())
    end
    local attr = {}
    for k, v in pairs(self.attr) do
        attr[k] = v
    end
    return nxml.new_element(self.name, attr, children)
end

---Allows you to have an xml element which represents a file, with changes made in the xml element reflecting in the file when you exit the `edit_file()` scope.
---Use like:
---```lua
---for content in nxml.edit_file("data/entities/animals/boss_centipede/boss_centipede.xml") do
---	content:first_of("DamageModelComponent"):set("hp", 2)
---end
----- Kolmis file is edited once we exit the for loop.
---```
---@param file str
---@param read (fun(filename: str): str)? `ModTextFileGetContent`
---@param write fun(filename: str, content: str)? `ModTextFileSetContent`
---@return fun(): element?
function nxml.edit_file(file, read, write)
    read = read or ModTextFileGetContent
    write = write or ModTextFileSetContent
    local first_time = true
    local tree = nxml.parse_file(file, read)
    return function()
        if not first_time then
            write(file, tostring(tree))
            return
        end
        first_time = false
        return tree
    end
end

---Parses a file. This is noita specific as it uses `ModTextFileGetContent`, but if you pass your own read function you can use it in a standalone context.
---@param file str
---@param read (fun(filename: str): str)? `ModTextFileGetContent`
---@return element
function nxml.parse_file(file, read)
    read = read or ModTextFileGetContent
    local content = read(file)
    local tok = new_tokenizer(content)
    local parser = new_parser(tok, nxml.error_handler)

    local elem = parser:parse_element(false)

    if not elem or (elem.errors and #elem.errors > 0) then
        error("parser encountered errors")
    end

    return elem
end

---The primary nxml function, converts nxml source into an element.
---Note it is the content not the filename, use `nxml.parse_file()` to parse by filename.
---@param data str
---@return element
function nxml.parse(data)
    local tok = new_tokenizer(data)
    local parser = new_parser(tok, nxml.error_handler)

    local elem = parser:parse_element(false)

    if not elem or (elem.errors and #elem.errors > 0) then
        error("parser encountered errors")
    end

    return elem
end

---This parses xml files with multiple base nodes, useful for biome xmls.
---Exaample file:
---```xml
---<A />
---<B />
---<C />
---```
---@param data str
---@return element[]
function nxml.parse_many(data)
    local tok = new_tokenizer(data)
    local parser = new_parser(tok, nxml.error_handler)

    local elems = parser:parse_elements()

    for i = 1, #elems do
        local elem = elems[i]

        if elem.errors and #elem.errors > 0 then
            error("parser encountered errors")
        end
    end

    return elems
end

---Constructs an element with the given values, just a wrapper to set the metatable really.
---@param name str
---@param attrs table<str, any>? {}
---@param children element[]? {}
---@return element
function nxml.new_element(name, attrs, children)
    local attr = {}
    attrs = attrs or {}
    for k, v in pairs(attrs) do
        attr[k] = attr_value_to_str(v)
    end
    ---@type element
    local element = {
        name = name,
        attr = attr,
        children = children or {},
        errors = {},
        content = nil,
    }
    ---@diagnostic disable-next-line: return-type-mismatch
    return setmetatable(element, XML_ELEMENT_MT)
end

---Generally you should do tostring(elem) instead of calling this function.
---This function is just how it's implemented and is exposed for more customisation.
---@param elem element
---@param packed bool
---@param indent_char str? \t
---@param cur_indent str? '""'
---@return str
function nxml.tostring(elem, packed, indent_char, cur_indent)
    indent_char = indent_char or "\t"
    cur_indent = cur_indent or ""
    local s = "<" .. elem.name
    local self_closing = #elem.children == 0 and (not elem.content or #elem.content == 0)

    for k, v in pairs(elem.attr) do
        s = s .. " " .. k .. '="' .. attr_value_to_str(v) .. '"'
    end

    if self_closing then
        s = s .. " />"
        return s
    end

    s = s .. ">"

    local deeper_indent = cur_indent .. indent_char

    if elem.content and #elem.content ~= 0 then
        if not packed then
            s = s .. "\n" .. deeper_indent
        end
        s = s .. elem:text()
    end

    if not packed then
        s = s .. "\n"
    end

    for _, v in ipairs(elem.children) do
        if not packed then
            s = s .. deeper_indent
        end
        s = s .. nxml.tostring(v, packed, indent_char, deeper_indent)
        if not packed then
            s = s .. "\n"
        end
    end

    s = s .. cur_indent .. "</" .. elem.name .. ">"

    return s
end

return nxml
