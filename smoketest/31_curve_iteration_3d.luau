eguidev.fixture("spacecurve.3d")
local vp = eguidev.root
eguidev.widget("pane.3d.curve"):expect({ visible = true })

local function expect_3d_canvas_painted()
    eguidev.widget("pane.3d.canvas"):expect({ visible = true })
    eguidev.widget("pane.3d.canvas"):expect({ painted = { min_colors = 3 } })
end

local curve = eguidev.widget("pane.3d.curve")
local state = assert(curve:state())
local options = state.options :: { string }
assert(#options >= 6, "expected at least 6 curve options in 3D, got " .. #options)
expect_3d_canvas_painted()

for i = 2, #options do
    curve:set_value(i - 1)
    eguidev.widget("pane.3d.curve"):wait(function(w)
        return w ~= nil and w.value == i - 1
    end)
    expect_3d_canvas_painted()
end

-- Verify all 3D sizes render for the default curve
eguidev.fixture("spacecurve.3d")
vp = eguidev.root
eguidev.widget("pane.3d.size"):expect({ visible = true })

local size = eguidev.widget("pane.3d.size")
local size_state = assert(size:state())
local size_options = size_state.options :: { string }
assert(#size_options >= 4, "expected at least 4 3D size options")

for i in ipairs(size_options) do
    size:set_value(i - 1)
    eguidev.widget("pane.3d.size"):wait(function(w)
        return w ~= nil and w.value == i - 1
    end)
    expect_3d_canvas_painted()
end
