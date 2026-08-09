eguidev.fixture("spacecurve.default")
local vp = eguidev.root
eguidev.widget("pane.2d.curve"):expect({ visible = true })

local function expect_2d_canvas_painted()
    eguidev.widget("pane.2d.canvas"):expect({ visible = true })
    eguidev.widget("pane.2d.canvas"):expect({ painted = { min_colors = 4 } })
end

local curve = eguidev.widget("pane.2d.curve")
local state = assert(curve:state())
local options = state.options :: { string }
assert(#options >= 6, "expected at least 6 curve options, got " .. #options)
expect_2d_canvas_painted()

for i = 2, #options do
    curve:set_value(i - 1)
    eguidev.widget("pane.2d.curve"):wait(function(w)
        return w ~= nil and w.value == i - 1
    end)
    expect_2d_canvas_painted()
end

-- Verify all sizes render for the default curve
eguidev.fixture("spacecurve.default")
vp = eguidev.root
eguidev.widget("pane.2d.size"):expect({ visible = true })

local size = eguidev.widget("pane.2d.size")
local size_state = assert(size:state())
local size_options = size_state.options :: { string }
assert(#size_options >= 4, "expected at least 4 size options")

for i in ipairs(size_options) do
    size:set_value(i - 1)
    eguidev.widget("pane.2d.size"):wait(function(w)
        return w ~= nil and w.value == i - 1
    end)
    if i == 1 then
        eguidev.widget("pane.2d.canvas"):expect({ visible = true })
    else
        expect_2d_canvas_painted()
    end
end
