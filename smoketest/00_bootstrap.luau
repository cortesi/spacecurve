local available = eguidev.fixtures()
assert(#available >= 5, "spacecurve should advertise the initial fixture set")

local fixture_names = {}
for _, fixture_info in ipairs(available) do
    fixture_names[fixture_info.name] = true
    if fixture_info.name == "spacecurve.default" then
        assert(#fixture_info.ready >= 1, "spacecurve.default should advertise ready conditions")
    end
end

assert(fixture_names["spacecurve.default"], "spacecurve.default fixture should exist")
assert(fixture_names["spacecurve.settings.3d"], "spacecurve.settings.3d fixture should exist")

eguidev.fixture("spacecurve.default")
local vp = eguidev.root

eguidev.widget("app.title"):expect({ visible = true })
eguidev.widget("app.tab.2d"):expect({ visible = true })
eguidev.widget("pane.2d.curve"):expect({ visible = true })
eguidev.widget("pane.2d.size"):expect({ visible = true })
eguidev.widget("pane.2d.pause"):expect({ visible = true })

local curve = assert(eguidev.widget("pane.2d.curve"):state())
assert(curve.role == "combo_box", "2D curve selector should be a combo box")
assert(curve.options ~= nil, "2D curve selector should expose options")
assert(#(curve.options :: { string }) >= 1, "2D curve selector should not be empty")
