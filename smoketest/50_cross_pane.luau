eguidev.fixture("spacecurve.default")
local vp = eguidev.root
eguidev.widget("pane.2d.curve"):expect({ visible = true })

local function expect_2d_canvas_painted()
    eguidev.widget("pane.2d.canvas"):expect({ visible = true })
    eguidev.widget("pane.2d.canvas"):expect({ painted = { min_colors = 4 } })
end

local function expect_3d_canvas_painted()
    eguidev.widget("pane.3d.canvas"):expect({ visible = true })
    eguidev.widget("pane.3d.canvas"):expect({ painted = { min_colors = 3 } })
end

-- Select a non-default curve in 2D
local curve_2d = eguidev.widget("pane.2d.curve")
local options_2d = assert(curve_2d:state()).options :: { string }
assert(#options_2d >= 3, "need at least 3 curves for cross-pane test")

curve_2d:set_value(2)
eguidev.widget("pane.2d.curve"):wait(function(w)
    return w ~= nil and w.value == 2
end)
local chosen_name = options_2d[3]

-- Switch to 3D: curve name should sync
eguidev.widget("app.tab.3d"):click()
expect_3d_canvas_painted()

local curve_3d = assert(eguidev.widget("pane.3d.curve"):state())
local curve_3d_value = assert(curve_3d.value) :: number
local name_3d = (curve_3d.options :: { string })[curve_3d_value + 1]
assert(
    name_3d == chosen_name,
    "3D curve should sync from 2D: expected " .. chosen_name .. " got " .. tostring(name_3d)
)

-- Now change curve in 3D and switch back
local options_3d = curve_3d.options :: { string }
eguidev.widget("pane.3d.curve"):set_value(0)
eguidev.widget("pane.3d.curve"):wait(function(w)
    return w ~= nil and w.value == 0
end)
local chosen_3d = options_3d[1]

eguidev.widget("app.tab.2d"):click()
expect_2d_canvas_painted()

local after_2d = assert(eguidev.widget("pane.2d.curve"):state())
local after_2d_value = assert(after_2d.value) :: number
local after_name = (after_2d.options :: { string })[after_2d_value + 1]
assert(
    after_name == chosen_3d,
    "2D curve should sync from 3D: expected " .. chosen_3d .. " got " .. tostring(after_name)
)

-- Verify pause state toggles and persists across pane switches
eguidev.widget("pane.2d.pause"):click()
local paused_2d = eguidev.widget("pane.2d.pause"):wait(function(w)
    return w ~= nil and w.selected == false
end)
assert(paused_2d ~= nil, "2D pause button should remain present")
assert(paused_2d.selected == false, "pause should toggle off in 2D")

eguidev.widget("app.tab.3d"):click()
expect_3d_canvas_painted()

local paused_3d = assert(eguidev.widget("pane.3d.pause"):state())
assert(paused_3d.selected == false, "pause state should persist across pane switch")
