eguidev.fixture("spacecurve.default")
local vp = eguidev.root
eguidev.widget("pane.2d.curve.info_toggle"):expect({ visible = true })

-- Open the curve info panel
eguidev.widget("pane.2d.curve.info_toggle"):click()
eguidev.widget("pane.2d.curve.info_pane"):expect({ visible = true })

local info = assert(eguidev.widget("pane.2d.curve.info_pane"):state())
assert(info.visible, "info pane should be visible after toggle click")

-- Close by clicking the toggle again
eguidev.widget("pane.2d.curve.info_toggle"):click()
eguidev.widget("pane.2d.curve.info_pane"):expect({ present = false })

-- Open again, close with Escape
eguidev.widget("pane.2d.curve.info_toggle"):click()
eguidev.widget("pane.2d.curve.info_pane"):expect({ visible = true })

vp:key("Escape")
eguidev.widget("pane.2d.curve.info_pane"):expect({ present = false })

-- Verify info pane updates when curve changes
eguidev.widget("pane.2d.curve.info_toggle"):click()
eguidev.widget("pane.2d.curve.info_pane"):expect({ visible = true })

local curve = eguidev.widget("pane.2d.curve")
local options = assert(curve:state()).options :: { string }
assert(#options >= 2, "need at least 2 curves for info switch test")

curve:set_value(1)
eguidev.widget("pane.2d.curve"):wait(function(w)
    return w ~= nil and w.value == 1
end)

-- Info pane should still be visible after curve change
assert(
    assert(eguidev.widget("pane.2d.curve.info_pane"):state()).visible,
    "info pane should remain visible after switching curves"
)

-- Test 3D info panel
eguidev.fixture("spacecurve.3d")
vp = eguidev.root
eguidev.widget("pane.3d.curve.info_toggle"):expect({ visible = true })

eguidev.widget("pane.3d.curve.info_toggle"):click()
eguidev.widget("pane.3d.curve.info_pane"):expect({ visible = true })

assert(
    assert(eguidev.widget("pane.3d.curve.info_pane"):state()).visible,
    "3D info pane should be visible"
)

eguidev.widget("pane.3d.curve.info_toggle"):click()
eguidev.widget("pane.3d.curve.info_pane"):expect({ present = false })
