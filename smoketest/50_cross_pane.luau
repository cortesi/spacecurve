fixture("spacecurve.default")
local vp = root()
vp:wait_for_widget_visible("pane.2d.curve")

-- Select a non-default curve in 2D
local curve_2d = vp:widget_get("pane.2d.curve")
local options_2d = curve_2d:state().options :: { string }
assert(#options_2d >= 3, "need at least 3 curves for cross-pane test")

curve_2d:set_value(2)
vp:wait_for_widget("pane.2d.curve", function(w)
    return w.value == 2
end)
local chosen_name = options_2d[3]

-- Switch to 3D: curve name should sync
vp:widget_get("app.tab.3d"):click()
vp:wait_for_widget_visible("pane.3d.canvas")

local curve_3d = vp:widget_get("pane.3d.curve"):state()
local name_3d = (curve_3d.options :: { string })[curve_3d.value + 1]
assert(
    name_3d == chosen_name,
    "3D curve should sync from 2D: expected " .. chosen_name .. " got " .. tostring(name_3d)
)

-- Now change curve in 3D and switch back
local options_3d = curve_3d.options :: { string }
vp:widget_get("pane.3d.curve"):set_value(0)
vp:wait_for_widget("pane.3d.curve", function(w)
    return w.value == 0
end)
local chosen_3d = options_3d[1]

vp:widget_get("app.tab.2d"):click()
vp:wait_for_widget_visible("pane.2d.canvas")

local after_2d = vp:widget_get("pane.2d.curve"):state()
local after_name = (after_2d.options :: { string })[after_2d.value + 1]
assert(
    after_name == chosen_3d,
    "2D curve should sync from 3D: expected " .. chosen_3d .. " got " .. tostring(after_name)
)

-- Verify pause state toggles and persists across pane switches
vp:widget_get("pane.2d.pause"):click()
local paused_2d = vp:wait_for_widget("pane.2d.pause", function(w)
    return w.selected == false
end)
assert(paused_2d.selected == false, "pause should toggle off in 2D")

vp:widget_get("app.tab.3d"):click()
vp:wait_for_widget_visible("pane.3d.canvas")

local paused_3d = vp:widget_get("pane.3d.pause"):state()
assert(paused_3d.selected == false, "pause state should persist across pane switch")
