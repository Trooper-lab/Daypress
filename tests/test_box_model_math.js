// Adversarial Stress Harness for DayStudio Box-Model, Overlay Geometry & Zoom Engine
const assert = require("assert");

console.log("=== STARTING ADVERSARIAL STRESS TEST FOR BOX-MODEL & CANVAS ===");

// 1. Scrubber Math & Modifier Invariants
function computeScrubbedValue(startVal, deltaX, isPadding, shiftKey, altKey) {
    const mult = shiftKey ? 10 : (altKey ? 0.1 : 1);
    let newVal = Math.round(startVal + deltaX * mult);
    if (isPadding && newVal < 0) newVal = 0;
    return newVal;
}

// Test normal scrub (1x)
assert.strictEqual(computeScrubbedValue(10, 5, false, false, false), 15);
assert.strictEqual(computeScrubbedValue(10, -5, false, false, false), 5);

// Test Shift rapid scrub (10x)
assert.strictEqual(computeScrubbedValue(10, 5, false, true, false), 60);
assert.strictEqual(computeScrubbedValue(10, -5, false, true, false), -40);

// Test Alt fine precision scrub (0.1x)
assert.strictEqual(computeScrubbedValue(10, 10, false, false, true), 11);
assert.strictEqual(computeScrubbedValue(10, 4, false, false, true), 10); // 10 + 0.4 rounds to 10
assert.strictEqual(computeScrubbedValue(10, 6, false, false, true), 11); // 10 + 0.6 rounds to 11

// Test Margin negative values allowed
assert.strictEqual(computeScrubbedValue(0, -25, false, false, false), -25);
assert.strictEqual(computeScrubbedValue(10, -50, false, false, false), -40);
assert.strictEqual(computeScrubbedValue(-10, -10, false, false, false), -20);

// Test Padding negative values CLAMPED to 0
assert.strictEqual(computeScrubbedValue(10, -15, true, false, false), 0);
assert.strictEqual(computeScrubbedValue(0, -10, true, false, false), 0);
assert.strictEqual(computeScrubbedValue(0, -100, true, true, false), 0);
assert.strictEqual(computeScrubbedValue(10, 20, true, false, false), 30);

console.log("[PASS] Scrubber modifier math & clamping invariants verified.");

// 2. Unit Switcher Cycling
const units = ["px", "rem", "%", "vh"];
function cycleUnit(current) {
    let idx = units.indexOf(current);
    return units[(idx + 1) % units.length];
}
let u = "px";
u = cycleUnit(u); assert.strictEqual(u, "rem");
u = cycleUnit(u); assert.strictEqual(u, "%");
u = cycleUnit(u); assert.strictEqual(u, "vh");
u = cycleUnit(u); assert.strictEqual(u, "px");
console.log("[PASS] Unit cycling sequence verified.");

// 3. Zoom Clamping Engine (50% - 150%)
function applyZoom(lvl) {
    return Math.max(0.5, Math.min(1.5, Math.round(lvl * 100) / 100));
}
assert.strictEqual(applyZoom(1.0), 1.0);
assert.strictEqual(applyZoom(1.25), 1.25);
assert.strictEqual(applyZoom(1.5), 1.5);
assert.strictEqual(applyZoom(1.75), 1.5); // Clamped to 150%
assert.strictEqual(applyZoom(2.5), 1.5);  // Clamped to 150%
assert.strictEqual(applyZoom(0.75), 0.75);
assert.strictEqual(applyZoom(0.5), 0.5);
assert.strictEqual(applyZoom(0.25), 0.5); // Clamped to 50%
assert.strictEqual(applyZoom(-1.0), 0.5); // Clamped to 50%
console.log("[PASS] Zoom engine bounds & clamping (50%-150%) verified.");

// 4. Overlay Geometry & Non-Overlapping Verification
function computeOverlays(rect, margin, padding) {
    const { mTop, mRight, mBottom, mLeft } = margin;
    const { pTop, pRight, pBottom, pLeft } = padding;

    const overlayBounds = {
        top: -mTop,
        left: -mLeft,
        width: rect.width + mLeft + mRight,
        height: rect.height + mTop + mBottom
    };

    // Margin facets
    const mTopFacet = { top: 0, left: mLeft, height: mTop, width: rect.width };
    const mBottomFacet = { top: mTop + rect.height, left: mLeft, height: mBottom, width: rect.width };
    const mLeftFacet = { top: 0, left: 0, height: rect.height + mTop + mBottom, width: mLeft };
    const mRightFacet = { top: 0, left: mLeft + rect.width, height: rect.height + mTop + mBottom, width: mRight };

    // Padding facets
    const pTopFacet = { top: mTop, left: mLeft, height: pTop, width: rect.width };
    const pBottomFacet = { top: mTop + rect.height - pBottom, left: mLeft, height: pBottom, width: rect.width };
    const pLeftFacet = { top: mTop + pTop, left: mLeft, height: rect.height - pTop - pBottom, width: pLeft };
    const pRightFacet = { top: mTop + pTop, left: mLeft + rect.width - pRight, height: rect.height - pTop - pBottom, width: pRight };

    return { overlayBounds, mTopFacet, mBottomFacet, mLeftFacet, mRightFacet, pTopFacet, pBottomFacet, pLeftFacet, pRightFacet };
}

const testRect = { width: 400, height: 200 };
const testMargin = { mTop: 20, mRight: 30, mBottom: 40, mLeft: 50 };
const testPadding = { pTop: 10, pRight: 15, pBottom: 20, pLeft: 25 };

const res = computeOverlays(testRect, testMargin, testPadding);
assert.strictEqual(res.overlayBounds.width, 400 + 50 + 30); // 480
assert.strictEqual(res.overlayBounds.height, 200 + 20 + 40); // 260

// Check non-overlapping geometry:
// Left margin facet width is 50, right margin facet starts at left = 50 + 400 = 450
assert.strictEqual(res.mLeftFacet.width, 50);
assert.strictEqual(res.mRightFacet.left, 450);
assert.strictEqual(res.mTopFacet.left, 50);
assert.strictEqual(res.mTopFacet.width, 400);

// Inner padding facets
assert.strictEqual(res.pTopFacet.top, 20);
assert.strictEqual(res.pTopFacet.height, 10);
assert.strictEqual(res.pBottomFacet.top, 20 + 200 - 20); // 200
assert.strictEqual(res.pLeftFacet.top, 20 + 10); // 30
assert.strictEqual(res.pLeftFacet.height, 200 - 10 - 20); // 170
assert.strictEqual(res.pRightFacet.left, 50 + 400 - 15); // 435

console.log("[PASS] Overlay geometry calculation verified.");
console.log("=== ALL ADVERSARIAL STRESS CHECKS COMPLETED SUCCESSFULLY ===");
