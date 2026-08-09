-- Test animation state: pause/resume, snake overlay behavior
eguidev.fixture("spacecurve.default")
local vp = eguidev.root
eguidev.widget("pane.2d.pause"):expect({ visible = true })

local function expect_2d_canvas_painted()
    eguidev.widget("pane.2d.canvas"):expect({ visible = true })
    eguidev.widget("pane.2d.canvas"):expect({ painted = { min_colors = 4 } })
end

local function expect_3d_canvas_painted()
    eguidev.widget("pane.3d.canvas"):expect({ visible = true })
    eguidev.widget("pane.3d.canvas"):expect({ painted = { min_colors = 3 } })
end

-- Default fixture has animation paused
local pause = assert(eguidev.widget("pane.2d.pause"):state())
assert(pause.selected == true, "default fixture should start paused")

-- Resume animation
eguidev.widget("pane.2d.pause"):click()
local resumed = eguidev.widget("pane.2d.pause"):expect({ selected = false })
assert(resumed ~= nil, "pause button should remain present after resume")
assert(resumed.selected == false, "pause button should toggle to unpaused")

-- Pause again
eguidev.widget("pane.2d.pause"):click()
local paused = eguidev.widget("pane.2d.pause"):expect({ selected = true })
assert(paused ~= nil, "pause button should remain present after pausing")
assert(paused.selected == true, "pause button should toggle back to paused")

-- Test in 3D: resume, then switch to 2D and back
eguidev.fixture("spacecurve.3d")
vp = eguidev.root
eguidev.widget("pane.3d.pause"):expect({ visible = true })

local pause_3d = assert(eguidev.widget("pane.3d.pause"):state())
assert(pause_3d.selected == true, "3D fixture should start paused")

eguidev.widget("pane.3d.pause"):click()
eguidev.widget("pane.3d.pause"):expect({ selected = false })

-- Switch to 2D and back: pause state should persist (it's shared)
eguidev.widget("app.tab.2d"):click()
expect_2d_canvas_painted()
local pause_2d_after = assert(eguidev.widget("pane.2d.pause"):state())
assert(pause_2d_after.selected == false, "pause state should persist to 2D pane")

eguidev.widget("app.tab.3d"):click()
expect_3d_canvas_painted()
local pause_3d_after = assert(eguidev.widget("pane.3d.pause"):state())
assert(pause_3d_after.selected == false, "pause state should persist back to 3D")

-- Enable snake overlay via settings, verify checkbox state
eguidev.fixture("spacecurve.settings.2d")
vp = eguidev.root

local snake = assert(eguidev.widget("settings.snake_enabled"):state())
assert(snake.value == false, "snake should be disabled in fixture")

eguidev.widget("settings.snake_enabled"):set_value(true)
eguidev.widget("settings.snake_enabled"):expect({ value = true })

-- With snake enabled, change its parameters
eguidev.widget("settings.snake_length"):set_value(25)
eguidev.widget("settings.snake_length"):expect({ value = 25 })

eguidev.widget("settings.snake_speed"):set_value(100)
eguidev.widget("settings.snake_speed"):expect({ value = 100 })

-- Enable long jumps for both curve and snake
eguidev.widget("settings.curve_long_jumps"):set_value(true)
eguidev.widget("settings.curve_long_jumps"):expect({ value = true })

eguidev.widget("settings.snake_long_jumps"):set_value(true)
eguidev.widget("settings.snake_long_jumps"):expect({ value = true })

-- Verify all settings stuck
assert(assert(eguidev.widget("settings.snake_enabled"):state()).value == true)
assert(assert(eguidev.widget("settings.snake_length"):state()).value == 25)
assert(assert(eguidev.widget("settings.snake_speed"):state()).value == 100)
assert(assert(eguidev.widget("settings.curve_long_jumps"):state()).value == true)
assert(assert(eguidev.widget("settings.snake_long_jumps"):state()).value == true)
