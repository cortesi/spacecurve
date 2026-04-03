fixture("spacecurve.default")
local vp = root()
vp:wait_for_widget_visible("pane.2d.canvas")

vp:widget_get("app.tab.3d"):click()
vp:wait_for_widget_visible("pane.3d.curve")
vp:wait_for_widget_visible("pane.3d.size")
vp:wait_for_widget_visible("pane.3d.pause")
vp:wait_for_widget_visible("pane.3d.canvas")

vp:widget_get("pane.3d.pause"):click()
local resumed = vp:wait_for_widget("pane.3d.pause", function(widget)
    return widget.selected == false
end)
assert(resumed.selected == false, "pause button should report resumed state after click")

vp:widget_get("app.about.toggle"):click()
vp:wait_for_widget_visible("dialog.about")
vp:wait_for_widget_visible("dialog.about.heading")
vp:widget_get("dialog.about.close"):click()
vp:wait_for_widget_absent("dialog.about")

vp:widget_get("app.tab.2d"):click()
vp:wait_for_widget_visible("pane.2d.canvas")
