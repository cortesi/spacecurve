local function find_dump_widget(node: any, id: string): any?
    if node.id == id then
        return node
    end
    for _, child in ipairs(node.children or {}) do
        local found = find_dump_widget(child, id)
        if found ~= nil then
            return found
        end
    end
    return nil
end

eguidev.fixture("spacecurve.settings.2d")
local vp = eguidev.root
eguidev.widget("pane.2d.canvas"):expect({ visible = true })
eguidev.wait(function()
    return eguidev.diagnostic("render").pane == "2d"
end)

local two_d_render = eguidev.diagnostic("render") :: any
assert(two_d_render.overlays.settings_open == true, "render diagnostic should expose settings state")
assert(two_d_render.two_d.curve ~= "", "render diagnostic should expose the 2D curve")
assert(two_d_render.two_d.screen_points > 0, "render diagnostic should expose rendered 2D points")

local two_d_tree = eguidev.dump({ root = eguidev.widget("pane.2d"), fields = "full" }) :: any
assert(#two_d_tree.viewports == 1, "2D pane dump should include one viewport")
assert(#two_d_tree.viewports[1].widgets == 1, "2D pane dump should be rooted at pane.2d")

local two_d_pane = two_d_tree.viewports[1].widgets[1]
assert(two_d_pane.id == "pane.2d", "2D pane dump root should be pane.2d")
local two_d_canvas = find_dump_widget(two_d_pane, "pane.2d.canvas")
assert(two_d_canvas ~= nil, "2D pane dump should include the canvas")
assert(two_d_canvas.visible == true, "2D canvas should be visible in the dump")
assert(two_d_canvas.label == "2D canvas", "2D canvas label should be preserved")

local two_d_text = eguidev.dump_text({ root = eguidev.widget("pane.2d") })
assert(string.find(two_d_text, "pane.2d.canvas", 1, true) ~= nil, "text dump should include 2D canvas")

eguidev.fixture("spacecurve.settings.3d")
vp = eguidev.root
eguidev.widget("pane.3d.canvas"):expect({ visible = true })
eguidev.wait(function()
    return eguidev.diagnostic("render").pane == "3d"
end)

local three_d_render = eguidev.diagnostic("render") :: any
assert(three_d_render.overlays.settings_open == true, "render diagnostic should expose 3D settings state")
assert(three_d_render.three_d.curve ~= "", "render diagnostic should expose the 3D curve")
assert(three_d_render.three_d.screen_points > 0, "render diagnostic should expose rendered 3D points")

local three_d_tree = eguidev.dump({ root = eguidev.widget("pane.3d"), fields = "full" }) :: any
assert(#three_d_tree.viewports == 1, "3D pane dump should include one viewport")
assert(#three_d_tree.viewports[1].widgets == 1, "3D pane dump should be rooted at pane.3d")

local three_d_pane = three_d_tree.viewports[1].widgets[1]
assert(three_d_pane.id == "pane.3d", "3D pane dump root should be pane.3d")
local three_d_canvas = find_dump_widget(three_d_pane, "pane.3d.canvas")
assert(three_d_canvas ~= nil, "3D pane dump should include the canvas")
assert(three_d_canvas.visible == true, "3D canvas should be visible in the dump")
assert(three_d_canvas.label == "3D canvas", "3D canvas label should be preserved")

return {
    two_d_canvas = two_d_canvas.id,
    two_d_curve = two_d_render.two_d.curve,
    three_d_canvas = three_d_canvas.id,
    three_d_curve = three_d_render.three_d.curve,
}
