eguidev.fixture("spacecurve.about")
local vp = eguidev.root

eguidev.widget("dialog.about.heading"):expect({ visible = true })
eguidev.widget("dialog.about.subtitle"):expect({ visible = true })
eguidev.widget("dialog.about.version"):expect({ visible = true })
eguidev.widget("dialog.about.close"):expect({ visible = true })

local heading = assert(eguidev.widget("dialog.about.heading"):state())
assert(heading.label == "spacecurve", "heading should be 'spacecurve', got: " .. tostring(heading.label))

local subtitle = assert(eguidev.widget("dialog.about.subtitle"):state())
assert(
    subtitle.label == "Space-filling curve playground",
    "subtitle text mismatch: " .. tostring(subtitle.label)
)

local version = assert(eguidev.widget("dialog.about.version"):state())
assert(version.label ~= nil, "version label should not be nil")
assert(string.sub(version.label :: string, 1, 1) == "v", "version should start with 'v'")

-- Close with X button
eguidev.widget("dialog.about.close"):click()
eguidev.widget("dialog.about"):expect({ present = false })

-- Reopen via menu bar and close with Escape
eguidev.widget("app.about.toggle"):click()
eguidev.widget("dialog.about"):expect({ visible = true })

vp:key("Escape")
eguidev.widget("dialog.about"):expect({ present = false })
