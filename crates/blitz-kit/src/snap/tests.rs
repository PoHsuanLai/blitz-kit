use super::grid::DeviceGrid;

/// At 1.5 a device pixel is 2/3 of a logical one.
const GRID: DeviceGrid = DeviceGrid(1.5);

#[test]
fn a_coordinate_lands_on_a_device_pixel() {
    for (logical, device) in [(11.0, 17.0), (10.0, 15.0), (0.4, 1.0), (0.3, 0.0)] {
        assert_eq!(GRID.snap(logical) * 1.5, device, "{logical}");
    }
}

#[test]
fn a_border_is_whole_device_pixels_and_never_vanishes() {
    // Stylo's snapped 1 px border at 1.5 (40 of 60 app units), and one just under it.
    for (width, device) in [(2.0 / 3.0, 1.0), (0.66, 1.0), (1.0, 2.0), (0.1, 1.0)] {
        let got = f64::from(GRID.line(width)) * 1.5;
        assert!((got - device).abs() < 1e-5, "{width}: {got}");
    }
    assert_eq!(GRID.line(0.0), 0.0);
}

#[test]
fn adjacent_spans_share_their_edge() {
    let (first, second) = (GRID.span(10.3, 7.4), GRID.span(17.7, 5.0));
    let end = GRID.snap(10.3) + f64::from(first) + f64::from(second);
    assert!((end - GRID.snap(22.7)).abs() < 1e-5);
}
