fixture("spacecurve.default")
local vp = root()
vp:wait_for_widget_visible("pane.2d.curve")

local curve = vp:widget_get("pane.2d.curve")
local curve_state = curve:state()
assert(curve_state.options ~= nil, "curve selector options should be exposed")
assert(#(curve_state.options :: { string }) >= 2, "need at least two curves to switch")
curve:set_value(1)
vp:wait_for_widget("pane.2d.curve", function(widget)
    return widget.value == 1
end)

local size = vp:widget_get("pane.2d.size")
size:set_value(2)
vp:wait_for_widget("pane.2d.size", function(widget)
    return widget.value == 2
end)

vp:widget_get("pane.2d.settings.toggle"):click()
vp:wait_for_widget_visible("settings.curve_opacity")
vp:wait_for_widget_visible("settings.curve_long_jumps")
vp:wait_for_widget_visible("settings.snake_length")

vp:widget_get("settings.curve_opacity"):set_value(80)
vp:wait_for_widget("settings.curve_opacity", function(widget)
    return widget.value == 80
end)

vp:widget_get("settings.curve_long_jumps"):set_value(true)
vp:wait_for_widget("settings.curve_long_jumps", function(widget)
    return widget.value == true
end)

vp:widget_get("settings.snake_length"):set_value(12.5)
vp:wait_for_widget("settings.snake_length", function(widget)
    return widget.value == 12.5
end)

fixture("spacecurve.settings.3d")
vp = root()
vp:wait_for_widget_visible("pane.3d.canvas")
vp:widget_get("settings.spin_speed"):set_value(25)
vp:wait_for_widget("settings.spin_speed", function(widget)
    return widget.value == 25
end)
