eguidev.fixture("spacecurve.default")
local vp = eguidev.root
eguidev.widget("pane.2d.curve"):expect({ visible = true })

local function expect_3d_canvas_painted()
    eguidev.widget("pane.3d.canvas"):expect({ visible = true })
    eguidev.widget("pane.3d.canvas"):expect({ painted = { min_colors = 3 } })
end

local curve = eguidev.widget("pane.2d.curve")
local curve_state = assert(curve:state())
assert(curve_state.options ~= nil, "curve selector options should be exposed")
assert(#(curve_state.options :: { string }) >= 2, "need at least two curves to switch")
curve:set_value(1)
eguidev.widget("pane.2d.curve"):expect({ value = 1 })

local size = eguidev.widget("pane.2d.size")
size:set_value(2)
eguidev.widget("pane.2d.size"):expect({ value = 2 })

eguidev.widget("pane.2d.settings.toggle"):click()
eguidev.widget("settings.curve_opacity"):expect({ visible = true })
eguidev.widget("settings.curve_long_jumps"):expect({ visible = true })
eguidev.widget("settings.snake_length"):expect({ visible = true })

eguidev.widget("settings.curve_opacity"):set_value(80)
eguidev.widget("settings.curve_opacity"):expect({ value = 80 })

eguidev.widget("settings.curve_long_jumps"):set_value(true)
eguidev.widget("settings.curve_long_jumps"):expect({ value = true })

eguidev.widget("settings.snake_length"):set_value(12.5)
eguidev.widget("settings.snake_length"):expect({ value = 12.5 })

eguidev.fixture("spacecurve.settings.3d")
vp = eguidev.root
expect_3d_canvas_painted()
eguidev.widget("settings.spin_speed"):set_value(25)
eguidev.widget("settings.spin_speed"):expect({ value = 25 })
