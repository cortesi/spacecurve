fixture("spacecurve.3d")
local vp = root()
expect("pane.3d.curve", { visible = true })

local function expect_3d_canvas_painted()
    expect("pane.3d.canvas", { visible = true })
    expect_painted("pane.3d.canvas", 3)
end

local curve = vp:widget_get("pane.3d.curve")
local state = curve:state()
local options = state.options :: { string }
assert(#options >= 6, "expected at least 6 curve options in 3D, got " .. #options)
expect_3d_canvas_painted()

for i = 2, #options do
    local name = options[i]
    curve:set_value(i - 1)
    vp:wait_for_widget("pane.3d.curve", function(w)
        return w.value == i - 1
    end)
    expect_3d_canvas_painted()
end

-- Verify all 3D sizes render for the default curve
fixture("spacecurve.3d")
vp = root()
expect("pane.3d.size", { visible = true })

local size = vp:widget_get("pane.3d.size")
local size_state = size:state()
local size_options = size_state.options :: { string }
assert(#size_options >= 4, "expected at least 4 3D size options")

for i in ipairs(size_options) do
    size:set_value(i - 1)
    vp:wait_for_widget("pane.3d.size", function(w)
        return w.value == i - 1
    end)
    expect_3d_canvas_painted()
end
