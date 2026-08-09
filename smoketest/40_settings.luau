eguidev.fixture("spacecurve.settings.2d")
local vp = eguidev.root

-- Verify all expected settings widgets are present
eguidev.widget("settings.curve_opacity"):expect({ visible = true })
eguidev.widget("settings.curve_long_jumps"):expect({ visible = true })
eguidev.widget("settings.snake_long_jumps"):expect({ visible = true })
eguidev.widget("settings.snake_length"):expect({ visible = true })
eguidev.widget("settings.snake_speed"):expect({ visible = true })

-- Verify slider ranges
local opacity = assert(eguidev.widget("settings.curve_opacity"):state())
local opacity_range = assert(opacity.range, "curve_opacity should expose its range")
assert(opacity.role == "slider", "curve_opacity should be a slider")
assert(opacity_range.min == 0, "opacity min should be 0")
assert(opacity_range.max == 100, "opacity max should be 100")

local snake_length = assert(eguidev.widget("settings.snake_length"):state())
local snake_length_range = assert(snake_length.range, "snake_length should expose its range")
assert(snake_length.role == "slider", "snake_length should be a slider")
assert(snake_length_range.min == 0, "snake_length min should be 0")
assert(snake_length_range.max == 50, "snake_length max should be 50")

local snake_speed = assert(eguidev.widget("settings.snake_speed"):state())
local snake_speed_range = assert(snake_speed.range, "snake_speed should expose its range")
assert(snake_speed.role == "slider", "snake_speed should be a slider")
assert(snake_speed_range.min == 1, "snake_speed min should be 1")
assert(snake_speed_range.max == 200, "snake_speed max should be 200")

-- Test slider set_value round-trips at boundaries
eguidev.widget("settings.curve_opacity"):set_value(0)
eguidev.widget("settings.curve_opacity"):expect({ value = 0 })

eguidev.widget("settings.curve_opacity"):set_value(100)
eguidev.widget("settings.curve_opacity"):expect({ value = 100 })

eguidev.widget("settings.snake_length"):set_value(0)
eguidev.widget("settings.snake_length"):expect({ value = 0 })

eguidev.widget("settings.snake_length"):set_value(50)
eguidev.widget("settings.snake_length"):expect({ value = 50 })

-- Test checkbox toggles round-trip
local checkboxes = {
    "settings.curve_long_jumps",
    "settings.snake_long_jumps",
    "settings.snake_enabled",
}
for _, id in ipairs(checkboxes) do
    local initial = assert(eguidev.widget(id):state()).value
    assert(type(initial) == "boolean", id .. " should expose a boolean value")

    eguidev.widget(id):set_value(not initial)
    eguidev.widget(id):expect({ value = not initial })

    eguidev.widget(id):set_value(initial)
    eguidev.widget(id):expect({ value = initial })
end

-- Verify spin_speed is NOT present in 2D settings
local all = vp:widgets()
for _, w in ipairs(all) do
    local s = assert(w:state())
    assert(s.id ~= "settings.spin_speed", "spin_speed should not appear in 2D settings")
end

-- Verify spin_speed IS present in 3D settings
eguidev.fixture("spacecurve.settings.3d")
vp = eguidev.root

local spin = assert(eguidev.widget("settings.spin_speed"):state())
local spin_range = assert(spin.range, "spin_speed should expose its range")
assert(spin.role == "slider", "spin_speed should be a slider")
assert(spin_range.min == 0, "spin_speed min should be 0")
assert(spin_range.max == 100, "spin_speed max should be 100")

eguidev.widget("settings.spin_speed"):set_value(0)
eguidev.widget("settings.spin_speed"):expect({ value = 0 })

eguidev.widget("settings.spin_speed"):set_value(100)
eguidev.widget("settings.spin_speed"):expect({ value = 100 })
