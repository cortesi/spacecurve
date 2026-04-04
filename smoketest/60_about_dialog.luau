fixture("spacecurve.about")
local vp = root()

vp:wait_for_widget_visible("dialog.about.heading")
vp:wait_for_widget_visible("dialog.about.subtitle")
vp:wait_for_widget_visible("dialog.about.version")
vp:wait_for_widget_visible("dialog.about.close")

local heading = vp:widget_get("dialog.about.heading"):state()
assert(heading.label == "spacecurve", "heading should be 'spacecurve', got: " .. tostring(heading.label))

local subtitle = vp:widget_get("dialog.about.subtitle"):state()
assert(
    subtitle.label == "Space-filling curve playground",
    "subtitle text mismatch: " .. tostring(subtitle.label)
)

local version = vp:widget_get("dialog.about.version"):state()
assert(version.label ~= nil, "version label should not be nil")
assert(string.sub(version.label :: string, 1, 1) == "v", "version should start with 'v'")

-- Close with X button
vp:widget_get("dialog.about.close"):click()
vp:wait_for_widget_absent("dialog.about")

-- Reopen via menu bar and close with Escape
vp:widget_get("app.about.toggle"):click()
vp:wait_for_widget_visible("dialog.about")

vp:key("Escape")
vp:wait_for_widget_absent("dialog.about")
