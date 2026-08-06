fixture("spacecurve.default")
local vp = root()

local function expect_2d_canvas_painted()
    expect("pane.2d.canvas", { visible = true })
    expect_painted("pane.2d.canvas", 4)
end

local function expect_3d_canvas_painted()
    expect("pane.3d.canvas", { visible = true })
    expect_painted("pane.3d.canvas", 3)
end

vp:widget_get("app.tab.3d"):click()
expect("pane.3d.curve", { visible = true })
expect("pane.3d.size", { visible = true })
expect("pane.3d.pause", { visible = true })
expect_3d_canvas_painted()

vp:widget_get("pane.3d.pause"):click()
local resumed = vp:wait_for_widget("pane.3d.pause", function(widget)
    return widget.selected == false
end)
assert(resumed.selected == false, "pause button should report resumed state after click")

vp:widget_get("app.about.toggle"):click()
expect("dialog.about", { visible = true })
expect("dialog.about.heading", { visible = true })
vp:widget_get("dialog.about.close"):click()
expect_absent("dialog.about")

vp:widget_get("app.tab.2d"):click()
expect_2d_canvas_painted()
