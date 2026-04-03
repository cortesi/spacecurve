-- Verify all fixtures can be applied repeatedly
local fixture_names = {
    "spacecurve.default",
    "spacecurve.3d",
    "spacecurve.about",
    "spacecurve.settings.2d",
    "spacecurve.settings.3d",
}

local function wait_for_fixture_ready(vp, name)
    if name == "spacecurve.default" then
        vp:wait_for_widget_visible("pane.2d.canvas")
    elseif name == "spacecurve.3d" then
        vp:wait_for_widget_visible("pane.3d.canvas")
    elseif name == "spacecurve.about" then
        vp:wait_for_widget_visible("dialog.about")
    elseif name == "spacecurve.settings.2d" then
        vp:wait_for_widget_visible("settings.snake_enabled")
    elseif name == "spacecurve.settings.3d" then
        vp:wait_for_widget_visible("settings.spin_speed")
    end
end

for _, name in ipairs(fixture_names) do
    fixture(name)
    local vp = root()
    wait_for_fixture_ready(vp, name)
    local widgets = vp:widget_list()
    assert(#widgets > 0, "fixture " .. name .. " should produce visible widgets")
end

-- Second pass proves fixtures can be reapplied
for _, name in ipairs(fixture_names) do
    fixture(name)
    local vp = root()
    wait_for_fixture_ready(vp, name)
    local widgets = vp:widget_list()
    assert(#widgets > 0, "fixture " .. name .. " should still work on second application")
end

-- Verify rapid alternation between fixtures
for _ = 1, 5 do
    fixture("spacecurve.default")
    root():wait_for_widget_visible("pane.2d.canvas")
    fixture("spacecurve.3d")
    root():wait_for_widget_visible("pane.3d.canvas")
end
