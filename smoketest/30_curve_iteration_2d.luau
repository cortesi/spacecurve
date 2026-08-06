fixture("spacecurve.default")
local vp = root()
expect("pane.2d.curve", { visible = true })

local function expect_2d_canvas_painted()
    expect("pane.2d.canvas", { visible = true })
    expect_painted("pane.2d.canvas", 4)
end

local curve = vp:widget_get("pane.2d.curve")
local state = curve:state()
local options = state.options :: { string }
assert(#options >= 6, "expected at least 6 curve options, got " .. #options)
expect_2d_canvas_painted()

for i = 2, #options do
    local name = options[i]
    curve:set_value(i - 1)
    vp:wait_for_widget("pane.2d.curve", function(w)
        return w.value == i - 1
    end)
    expect_2d_canvas_painted()
end

-- Verify all sizes render for the default curve
fixture("spacecurve.default")
vp = root()
expect("pane.2d.size", { visible = true })

local size = vp:widget_get("pane.2d.size")
local size_state = size:state()
local size_options = size_state.options :: { string }
assert(#size_options >= 4, "expected at least 4 size options")

for i in ipairs(size_options) do
    size:set_value(i - 1)
    vp:wait_for_widget("pane.2d.size", function(w)
        return w.value == i - 1
    end)
    if i == 1 then
        expect("pane.2d.canvas", { visible = true })
    else
        expect_2d_canvas_painted()
    end
end
