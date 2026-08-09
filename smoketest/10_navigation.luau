eguidev.fixture("spacecurve.default")
local vp = eguidev.root

local function expect_2d_canvas_painted()
    eguidev.widget("pane.2d.canvas"):expect({ visible = true })
    eguidev.widget("pane.2d.canvas"):expect({ painted = { min_colors = 4 } })
end

local function expect_3d_canvas_painted()
    eguidev.widget("pane.3d.canvas"):expect({ visible = true })
    eguidev.widget("pane.3d.canvas"):expect({ painted = { min_colors = 3 } })
end

eguidev.widget("app.tab.3d"):click()
eguidev.widget("pane.3d.curve"):expect({ visible = true })
eguidev.widget("pane.3d.size"):expect({ visible = true })
eguidev.widget("pane.3d.pause"):expect({ visible = true })
expect_3d_canvas_painted()

eguidev.widget("pane.3d.pause"):click()
local resumed = eguidev.widget("pane.3d.pause"):wait(function(widget)
    return widget ~= nil and widget.selected == false
end)
assert(resumed ~= nil, "pause button should remain present after click")
assert(resumed.selected == false, "pause button should report resumed state after click")

eguidev.widget("app.about.toggle"):click()
eguidev.widget("dialog.about"):expect({ visible = true })
eguidev.widget("dialog.about.heading"):expect({ visible = true })
eguidev.widget("dialog.about.close"):click()
eguidev.widget("dialog.about"):expect({ present = false })

eguidev.widget("app.tab.2d"):click()
expect_2d_canvas_painted()
