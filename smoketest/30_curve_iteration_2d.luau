fixture("spacecurve.default")
local vp = root()
vp:wait_for_widget_visible("pane.2d.curve")

local curve = vp:widget_get("pane.2d.curve")
local state = curve:state()
local options = state.options :: { string }
assert(#options >= 6, "expected at least 6 curve options, got " .. #options)

for i, name in ipairs(options) do
    curve:set_value(i - 1)
    vp:wait_for_widget("pane.2d.curve", function(w)
        return w.value == i - 1
    end)
    assert(
        vp:widget_get("pane.2d.canvas"):state().visible,
        "canvas should remain visible after selecting " .. name
    )
end

-- Verify all sizes render for the default curve
fixture("spacecurve.default")
vp = root()
vp:wait_for_widget_visible("pane.2d.size")

local size = vp:widget_get("pane.2d.size")
local size_state = size:state()
local size_options = size_state.options :: { string }
assert(#size_options >= 4, "expected at least 4 size options")

for i in ipairs(size_options) do
    size:set_value(i - 1)
    vp:wait_for_widget("pane.2d.size", function(w)
        return w.value == i - 1
    end)
    assert(
        vp:widget_get("pane.2d.canvas"):state().visible,
        "canvas should remain visible at size index " .. (i - 1)
    )
end
