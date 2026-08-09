-- Stress test: rapid fixture cycling and settings manipulation.

-- Phase 1: Rapid fixture cycling
eguidev.fixture("spacecurve.default")
eguidev.fixture("spacecurve.3d")
eguidev.fixture("spacecurve.about")
eguidev.fixture("spacecurve.settings.2d")
eguidev.fixture("spacecurve.settings.3d")
eguidev.fixture("spacecurve.default")

-- Phase 2: Curve switching in 2D
local vp = eguidev.root
eguidev.widget("pane.2d.curve"):expect({ visible = true })
local function expect_2d_canvas_painted()
    eguidev.widget("pane.2d.canvas"):expect({ visible = true })
    eguidev.widget("pane.2d.canvas"):expect({ painted = { min_colors = 4 } })
end

local curve = eguidev.widget("pane.2d.curve")
local options = assert(curve:state()).options :: { string }

for i = 1, #options do
    curve:set_value(i - 1)
    eguidev.widget("pane.2d.curve"):wait(function(w)
        return w ~= nil and w.value == i - 1
    end)
end

-- Phase 3: Settings manipulation with animation
eguidev.fixture("spacecurve.settings.2d")
vp = eguidev.root

eguidev.widget("settings.snake_enabled"):set_value(true)
eguidev.widget("settings.snake_enabled"):wait(function(w)
    return w ~= nil and w.value == true
end)

eguidev.widget("settings.snake_length"):set_value(25)
eguidev.widget("settings.snake_length"):wait(function(w)
    return w ~= nil and w.value == 25
end)

eguidev.widget("settings.curve_opacity"):set_value(50)
eguidev.widget("settings.curve_opacity"):wait(function(w)
    return w ~= nil and w.value == 50
end)

-- Phase 4: Final fixture to confirm everything still works
eguidev.fixture("spacecurve.default")
vp = eguidev.root
expect_2d_canvas_painted()
