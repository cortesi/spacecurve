-- Stress test: rapid fixture cycling and settings manipulation.

-- Phase 1: Rapid fixture cycling
fixture("spacecurve.default")
fixture("spacecurve.3d")
fixture("spacecurve.about")
fixture("spacecurve.settings.2d")
fixture("spacecurve.settings.3d")
fixture("spacecurve.default")

-- Phase 2: Curve switching in 2D
local vp = root()
vp:wait_for_widget_visible("pane.2d.curve")
local curve = vp:widget_get("pane.2d.curve")
local options = curve:state().options :: { string }

for i = 1, #options do
    curve:set_value(i - 1)
    vp:wait_for_widget("pane.2d.curve", function(w)
        return w.value == i - 1
    end)
end

-- Phase 3: Settings manipulation with animation
fixture("spacecurve.settings.2d")
vp = root()
vp:wait_for_widget_visible("settings.snake_enabled")

vp:widget_get("settings.snake_enabled"):set_value(true)
vp:wait_for_widget("settings.snake_enabled", function(w)
    return w.value == true
end)

vp:widget_get("settings.snake_length"):set_value(25)
vp:wait_for_widget("settings.snake_length", function(w)
    return w.value == 25
end)

vp:widget_get("settings.curve_opacity"):set_value(50)
vp:wait_for_widget("settings.curve_opacity", function(w)
    return w.value == 50
end)

-- Phase 4: Final fixture to confirm everything still works
fixture("spacecurve.default")
vp = root()
vp:wait_for_widget_visible("pane.2d.canvas")
assert(
    vp:widget_get("pane.2d.canvas"):state().visible,
    "canvas should be visible after stress test"
)
