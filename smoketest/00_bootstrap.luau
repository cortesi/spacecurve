local available = fixtures()
assert(#available >= 5, "spacecurve should advertise the initial fixture set")

local fixture_names = {}
for _, fixture_info in ipairs(available) do
    fixture_names[fixture_info.name] = true
end

assert(fixture_names["spacecurve.default"], "spacecurve.default fixture should exist")
assert(fixture_names["spacecurve.settings.3d"], "spacecurve.settings.3d fixture should exist")

fixture("spacecurve.default")
local vp = root()

vp:wait_for_widget_visible("app.title")
vp:wait_for_widget_visible("app.tab.2d")
vp:wait_for_widget_visible("pane.2d.curve")
vp:wait_for_widget_visible("pane.2d.size")
vp:wait_for_widget_visible("pane.2d.pause")
vp:wait_for_widget_visible("pane.2d.canvas")

local curve = vp:widget_get("pane.2d.curve"):state()
assert(curve.role == "combo_box", "2D curve selector should be a combo box")
assert(curve.options ~= nil, "2D curve selector should expose options")
assert(#(curve.options :: { string }) >= 1, "2D curve selector should not be empty")

return {
    fixtures = #available,
    default_curve = ((curve.options :: { string })[(curve.value :: number) + 1]),
}
