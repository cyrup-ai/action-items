use action_items_ecs_ui::visibility::BezierEasing;

#[test]
fn test_bezier_easing_edge_cases() {
    let ease_in = BezierEasing::EASE_IN;

    // Edge case: x = 0.0 should return 0.0
    assert_eq!(ease_in.apply(0.0), 0.0);

    // Edge case: x = 1.0 should return 1.0
    assert_eq!(ease_in.apply(1.0), 1.0);
}

#[test]
fn test_bezier_easing_ease_in() {
    let ease_in = BezierEasing::EASE_IN;

    // EASE_IN is (0.42, 0.0, 1.0, 1.0)
    // At x=0.5, should return a value between 0.0 and 1.0
    let result = ease_in.apply(0.5);
    assert!(result >= 0.0 && result <= 1.0, "Result {} should be in [0.0, 1.0]", result);

    // EASE_IN should start slow, so at x=0.5 should be less than 0.5 (below linear)
    assert!(result < 0.5, "EASE_IN(0.5) = {} should be < 0.5 (slow start)", result);
}

#[test]
fn test_bezier_easing_ease_out() {
    let ease_out = BezierEasing::EASE_OUT;

    // Edge cases
    assert_eq!(ease_out.apply(0.0), 0.0);
    assert_eq!(ease_out.apply(1.0), 1.0);

    // EASE_OUT is (0.0, 0.0, 0.58, 1.0)
    // At x=0.5, should return a value between 0.0 and 1.0
    let result = ease_out.apply(0.5);
    assert!(result >= 0.0 && result <= 1.0, "Result {} should be in [0.0, 1.0]", result);

    // EASE_OUT should start fast, so at x=0.5 should be greater than 0.5 (above linear)
    assert!(result > 0.5, "EASE_OUT(0.5) = {} should be > 0.5 (fast start)", result);
}

#[test]
fn test_bezier_easing_ease_in_out() {
    let ease_in_out = BezierEasing::EASE_IN_OUT;

    // Edge cases
    assert_eq!(ease_in_out.apply(0.0), 0.0);
    assert_eq!(ease_in_out.apply(1.0), 1.0);

    // EASE_IN_OUT is (0.42, 0.0, 0.58, 1.0)
    // At x=0.5, should return approximately 0.5 (S-curve)
    let result = ease_in_out.apply(0.5);
    assert!(result >= 0.0 && result <= 1.0, "Result {} should be in [0.0, 1.0]", result);

    // Should be close to 0.5 for symmetric ease-in-out
    assert!((result - 0.5).abs() < 0.1, "EASE_IN_OUT(0.5) = {} should be close to 0.5", result);
}

#[test]
fn test_bezier_easing_linear() {
    let linear = BezierEasing::new(0.0, 0.0, 1.0, 1.0);

    // Linear curve should return x
    assert_eq!(linear.apply(0.0), 0.0);
    assert_eq!(linear.apply(0.5), 0.5);
    assert_eq!(linear.apply(1.0), 1.0);
}

#[test]
fn test_bezier_easing_custom() {
    let custom = BezierEasing::new(0.25, 0.1, 0.25, 1.0);

    // Edge cases
    assert_eq!(custom.apply(0.0), 0.0);
    assert_eq!(custom.apply(1.0), 1.0);

    // Values in between should be valid
    let result = custom.apply(0.5);
    assert!(result >= 0.0 && result <= 1.0, "Result {} should be in [0.0, 1.0]", result);
}

#[test]
fn test_bezier_easing_monotonic() {
    let ease_in = BezierEasing::EASE_IN;

    // Test that curve is monotonically increasing
    let mut prev = 0.0;
    for i in 0..=10 {
        let x = i as f32 / 10.0;
        let y = ease_in.apply(x);
        assert!(y >= prev, "Curve should be monotonically increasing: {} < {} at x={}", y, prev, x);
        prev = y;
    }
}
