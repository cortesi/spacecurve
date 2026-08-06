fixture("spacecurve.default")
local vp = root()
expect("pane.2d.curve", { visible = true })

local function expect_3d_canvas_painted()
    expect("pane.3d.canvas", { visible = true })
    expect_painted("pane.3d.canvas", 3)
end

local curve = vp:widget_get("pane.2d.curve")
local curve_state = curve:state()
assert(curve_state.options ~= nil, "curve selector options should be exposed")
assert(#(curve_state.options :: { string }) >= 2, "need at least two curves to switch")
curve:set_value(1)
expect("pane.2d.curve", { value = 1 })

local size = vp:widget_get("pane.2d.size")
size:set_value(2)
expect("pane.2d.size", { value = 2 })

vp:widget_get("pane.2d.settings.toggle"):click()
expect("settings.curve_opacity", { visible = true })
expect("settings.curve_long_jumps", { visible = true })
expect("settings.snake_length", { visible = true })

vp:widget_get("settings.curve_opacity"):set_value(80)
expect("settings.curve_opacity", { value = 80 })

vp:widget_get("settings.curve_long_jumps"):set_value(true)
expect("settings.curve_long_jumps", { value = true })

vp:widget_get("settings.snake_length"):set_value(12.5)
expect("settings.snake_length", { value = 12.5 })

fixture("spacecurve.settings.3d")
vp = root()
expect_3d_canvas_painted()
vp:widget_get("settings.spin_speed"):set_value(25)
expect("settings.spin_speed", { value = 25 })
