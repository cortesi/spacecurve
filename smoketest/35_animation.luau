-- Test animation state: pause/resume, snake overlay behavior
fixture("spacecurve.default")
local vp = root()
expect("pane.2d.pause", { visible = true })

local function expect_2d_canvas_painted()
    expect("pane.2d.canvas", { visible = true })
    expect_painted("pane.2d.canvas", 4)
end

local function expect_3d_canvas_painted()
    expect("pane.3d.canvas", { visible = true })
    expect_painted("pane.3d.canvas", 3)
end

-- Default fixture has animation paused
local pause = vp:widget_get("pane.2d.pause"):state()
assert(pause.selected == true, "default fixture should start paused")

-- Resume animation
vp:widget_get("pane.2d.pause"):click()
local resumed = expect("pane.2d.pause", { selected = false })
assert(resumed.selected == false, "pause button should toggle to unpaused")

-- Pause again
vp:widget_get("pane.2d.pause"):click()
local paused = expect("pane.2d.pause", { selected = true })
assert(paused.selected == true, "pause button should toggle back to paused")

-- Test in 3D: resume, then switch to 2D and back
fixture("spacecurve.3d")
vp = root()
expect("pane.3d.pause", { visible = true })

local pause_3d = vp:widget_get("pane.3d.pause"):state()
assert(pause_3d.selected == true, "3D fixture should start paused")

vp:widget_get("pane.3d.pause"):click()
expect("pane.3d.pause", { selected = false })

-- Switch to 2D and back: pause state should persist (it's shared)
vp:widget_get("app.tab.2d"):click()
expect_2d_canvas_painted()
local pause_2d_after = vp:widget_get("pane.2d.pause"):state()
assert(pause_2d_after.selected == false, "pause state should persist to 2D pane")

vp:widget_get("app.tab.3d"):click()
expect_3d_canvas_painted()
local pause_3d_after = vp:widget_get("pane.3d.pause"):state()
assert(pause_3d_after.selected == false, "pause state should persist back to 3D")

-- Enable snake overlay via settings, verify checkbox state
fixture("spacecurve.settings.2d")
vp = root()

local snake = vp:widget_get("settings.snake_enabled"):state()
assert(snake.value == false, "snake should be disabled in fixture")

vp:widget_get("settings.snake_enabled"):set_value(true)
expect("settings.snake_enabled", { value = true })

-- With snake enabled, change its parameters
vp:widget_get("settings.snake_length"):set_value(25)
expect("settings.snake_length", { value = 25 })

vp:widget_get("settings.snake_speed"):set_value(100)
expect("settings.snake_speed", { value = 100 })

-- Enable long jumps for both curve and snake
vp:widget_get("settings.curve_long_jumps"):set_value(true)
expect("settings.curve_long_jumps", { value = true })

vp:widget_get("settings.snake_long_jumps"):set_value(true)
expect("settings.snake_long_jumps", { value = true })

-- Verify all settings stuck
assert(vp:widget_get("settings.snake_enabled"):state().value == true)
assert(vp:widget_get("settings.snake_length"):state().value == 25)
assert(vp:widget_get("settings.snake_speed"):state().value == 100)
assert(vp:widget_get("settings.curve_long_jumps"):state().value == true)
assert(vp:widget_get("settings.snake_long_jumps"):state().value == true)
