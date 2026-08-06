fixture("spacecurve.about")
local vp = root()

expect("dialog.about.heading", { visible = true })
expect("dialog.about.subtitle", { visible = true })
expect("dialog.about.version", { visible = true })
expect("dialog.about.close", { visible = true })

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
expect_absent("dialog.about")

-- Reopen via menu bar and close with Escape
vp:widget_get("app.about.toggle"):click()
expect("dialog.about", { visible = true })

vp:key("Escape")
expect_absent("dialog.about")
