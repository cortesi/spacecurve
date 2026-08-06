fixture("spacecurve.settings.2d")
local vp = root()

-- Verify all expected settings widgets are present
expect("settings.curve_opacity", { visible = true })
expect("settings.curve_long_jumps", { visible = true })
expect("settings.snake_long_jumps", { visible = true })
expect("settings.snake_length", { visible = true })
expect("settings.snake_speed", { visible = true })

-- Verify slider ranges
local opacity = vp:widget_get("settings.curve_opacity"):state()
assert(opacity.role == "slider", "curve_opacity should be a slider")
assert(opacity.range.min == 0, "opacity min should be 0")
assert(opacity.range.max == 100, "opacity max should be 100")

local snake_length = vp:widget_get("settings.snake_length"):state()
assert(snake_length.role == "slider", "snake_length should be a slider")
assert(snake_length.range.min == 0, "snake_length min should be 0")
assert(snake_length.range.max == 50, "snake_length max should be 50")

local snake_speed = vp:widget_get("settings.snake_speed"):state()
assert(snake_speed.role == "slider", "snake_speed should be a slider")
assert(snake_speed.range.min == 1, "snake_speed min should be 1")
assert(snake_speed.range.max == 200, "snake_speed max should be 200")

-- Test slider set_value round-trips at boundaries
vp:widget_get("settings.curve_opacity"):set_value(0)
expect("settings.curve_opacity", { value = 0 })

vp:widget_get("settings.curve_opacity"):set_value(100)
expect("settings.curve_opacity", { value = 100 })

vp:widget_get("settings.snake_length"):set_value(0)
expect("settings.snake_length", { value = 0 })

vp:widget_get("settings.snake_length"):set_value(50)
expect("settings.snake_length", { value = 50 })

-- Test checkbox toggles round-trip
local checkboxes = {
    "settings.curve_long_jumps",
    "settings.snake_long_jumps",
    "settings.snake_enabled",
}
for _, id in ipairs(checkboxes) do
    local initial = vp:widget_get(id):state().value

    vp:widget_get(id):set_value(not initial)
    expect(id, { value = not initial })

    vp:widget_get(id):set_value(initial)
    expect(id, { value = initial })
end

-- Verify spin_speed is NOT present in 2D settings
local all = vp:widget_list()
for _, w in ipairs(all) do
    local s = w:state()
    assert(s.id ~= "settings.spin_speed", "spin_speed should not appear in 2D settings")
end

-- Verify spin_speed IS present in 3D settings
fixture("spacecurve.settings.3d")
vp = root()

local spin = vp:widget_get("settings.spin_speed"):state()
assert(spin.role == "slider", "spin_speed should be a slider")
assert(spin.range.min == 0, "spin_speed min should be 0")
assert(spin.range.max == 100, "spin_speed max should be 100")

vp:widget_get("settings.spin_speed"):set_value(0)
expect("settings.spin_speed", { value = 0 })

vp:widget_get("settings.spin_speed"):set_value(100)
expect("settings.spin_speed", { value = 100 })
