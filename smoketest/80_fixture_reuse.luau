-- Verify all fixtures can be applied repeatedly
local fixture_names = {
    "spacecurve.default",
    "spacecurve.3d",
    "spacecurve.about",
    "spacecurve.settings.2d",
    "spacecurve.settings.3d",
}

for _, name in ipairs(fixture_names) do
    eguidev.fixture(name)
    local vp = eguidev.root
    local widgets = vp:widgets()
    assert(#widgets > 0, "fixture " .. name .. " should produce visible widgets")
end

-- Second pass proves fixtures can be reapplied
for _, name in ipairs(fixture_names) do
    eguidev.fixture(name)
    local vp = eguidev.root
    local widgets = vp:widgets()
    assert(#widgets > 0, "fixture " .. name .. " should still work on second application")
end

-- Verify rapid alternation between fixtures
for _ = 1, 5 do
    eguidev.fixture("spacecurve.default")
    eguidev.fixture("spacecurve.3d")
end
