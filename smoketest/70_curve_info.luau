fixture("spacecurve.default")
local vp = root()
expect("pane.2d.curve.info_toggle", { visible = true })

-- Open the curve info panel
vp:widget_get("pane.2d.curve.info_toggle"):click()
expect("pane.2d.curve.info_pane", { visible = true })

local info = vp:widget_get("pane.2d.curve.info_pane"):state()
assert(info.visible, "info pane should be visible after toggle click")

-- Close by clicking the toggle again
vp:widget_get("pane.2d.curve.info_toggle"):click()
expect_absent("pane.2d.curve.info_pane")

-- Open again, close with Escape
vp:widget_get("pane.2d.curve.info_toggle"):click()
expect("pane.2d.curve.info_pane", { visible = true })

vp:key("Escape")
expect_absent("pane.2d.curve.info_pane")

-- Verify info pane updates when curve changes
vp:widget_get("pane.2d.curve.info_toggle"):click()
expect("pane.2d.curve.info_pane", { visible = true })

local curve = vp:widget_get("pane.2d.curve")
local options = curve:state().options :: { string }
assert(#options >= 2, "need at least 2 curves for info switch test")

curve:set_value(1)
vp:wait_for_widget("pane.2d.curve", function(w)
    return w.value == 1
end)

-- Info pane should still be visible after curve change
assert(
    vp:widget_get("pane.2d.curve.info_pane"):state().visible,
    "info pane should remain visible after switching curves"
)

-- Test 3D info panel
fixture("spacecurve.3d")
vp = root()
expect("pane.3d.curve.info_toggle", { visible = true })

vp:widget_get("pane.3d.curve.info_toggle"):click()
expect("pane.3d.curve.info_pane", { visible = true })

assert(
    vp:widget_get("pane.3d.curve.info_pane"):state().visible,
    "3D info pane should be visible"
)

vp:widget_get("pane.3d.curve.info_toggle"):click()
expect_absent("pane.3d.curve.info_pane")
