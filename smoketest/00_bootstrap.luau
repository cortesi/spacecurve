local available = fixtures()
assert(#available >= 5, "spacecurve should advertise the initial fixture set")

local fixture_names = {}
for _, fixture_info in ipairs(available) do
    fixture_names[fixture_info.name] = true
    if fixture_info.name == "spacecurve.default" then
        assert(#fixture_info.anchors >= 1, "spacecurve.default should advertise readiness anchors")
    end
end

assert(fixture_names["spacecurve.default"], "spacecurve.default fixture should exist")
assert(fixture_names["spacecurve.settings.3d"], "spacecurve.settings.3d fixture should exist")

fixture("spacecurve.default")
local vp = root()

expect("app.title", { visible = true })
expect("app.tab.2d", { visible = true })
expect("pane.2d.curve", { visible = true })
expect("pane.2d.size", { visible = true })
expect("pane.2d.pause", { visible = true })

local curve = vp:widget_get("pane.2d.curve"):state()
assert(curve.role == "combo_box", "2D curve selector should be a combo box")
assert(curve.options ~= nil, "2D curve selector should expose options")
assert(#(curve.options :: { string }) >= 1, "2D curve selector should not be empty")
