-- Test animation state: pause/resume, snake overlay behavior
fixture("spacecurve.default")
local vp = root()
vp:wait_for_widget_visible("pane.2d.pause")

-- Default fixture has animation paused
local pause = vp:widget_get("pane.2d.pause"):state()
assert(pause.selected == true, "default fixture should start paused")

-- Resume animation
vp:widget_get("pane.2d.pause"):click()
local resumed = vp:wait_for_widget("pane.2d.pause", function(w)
    return w.selected == false
end)
assert(resumed.selected == false, "pause button should toggle to unpaused")

-- Pause again
vp:widget_get("pane.2d.pause"):click()
local paused = vp:wait_for_widget("pane.2d.pause", function(w)
    return w.selected == true
end)
assert(paused.selected == true, "pause button should toggle back to paused")

-- Test in 3D: resume, then switch to 2D and back
fixture("spacecurve.3d")
vp = root()
vp:wait_for_widget_visible("pane.3d.pause")

local pause_3d = vp:widget_get("pane.3d.pause"):state()
assert(pause_3d.selected == true, "3D fixture should start paused")

vp:widget_get("pane.3d.pause"):click()
vp:wait_for_widget("pane.3d.pause", function(w)
    return w.selected == false
end)

-- Switch to 2D and back: pause state should persist (it's shared)
vp:widget_get("app.tab.2d"):click()
vp:wait_for_widget_visible("pane.2d.canvas")
local pause_2d_after = vp:widget_get("pane.2d.pause"):state()
assert(pause_2d_after.selected == false, "pause state should persist to 2D pane")

vp:widget_get("app.tab.3d"):click()
vp:wait_for_widget_visible("pane.3d.canvas")
local pause_3d_after = vp:widget_get("pane.3d.pause"):state()
assert(pause_3d_after.selected == false, "pause state should persist back to 3D")

-- Enable snake overlay via settings, verify checkbox state
fixture("spacecurve.settings.2d")
vp = root()

local snake = vp:widget_get("settings.snake_enabled"):state()
assert(snake.value == false, "snake should be disabled in fixture")

vp:widget_get("settings.snake_enabled"):set_value(true)
vp:wait_for_widget("settings.snake_enabled", function(w)
    return w.value == true
end)

-- With snake enabled, change its parameters
vp:widget_get("settings.snake_length"):set_value(25)
vp:wait_for_widget("settings.snake_length", function(w)
    return w.value == 25
end)

vp:widget_get("settings.snake_speed"):set_value(100)
vp:wait_for_widget("settings.snake_speed", function(w)
    return w.value == 100
end)

-- Enable long jumps for both curve and snake
vp:widget_get("settings.curve_long_jumps"):set_value(true)
vp:wait_for_widget("settings.curve_long_jumps", function(w)
    return w.value == true
end)

vp:widget_get("settings.snake_long_jumps"):set_value(true)
vp:wait_for_widget("settings.snake_long_jumps", function(w)
    return w.value == true
end)

-- Verify all settings stuck
assert(vp:widget_get("settings.snake_enabled"):state().value == true)
assert(vp:widget_get("settings.snake_length"):state().value == 25)
assert(vp:widget_get("settings.snake_speed"):state().value == 100)
assert(vp:widget_get("settings.curve_long_jumps"):state().value == true)
assert(vp:widget_get("settings.snake_long_jumps"):state().value == true)
