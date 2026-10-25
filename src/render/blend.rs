pub fn blend(dst: [u8; 4], src: [u8; 4], coverage: u8) -> [u8; 4] {
    let alpha = src[3] as u16 * coverage as u16 / 255;
    let inverse = 255 - alpha;

    let channel = |source: u8, destination: u8| {
        ((source as u16 * alpha + destination as u16 * inverse + 127) / 255) as u8
    };

    [
        channel(src[0], dst[0]),
        channel(src[1], dst[1]),
        channel(src[2], dst[2]),
        (alpha + (dst[3] as u16 * inverse + 127) / 255).min(255) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_coverage_keeps_destination() {
        let dst = [10, 20, 30, 255];
        assert_eq!(blend(dst, [255, 255, 255, 255], 0), dst);
    }

    #[test]
    fn opaque_coverage_replaces_destination() {
        assert_eq!(
            blend([10, 20, 30, 255], [200, 100, 50, 255], 255),
            [200, 100, 50, 255]
        );
    }

    #[test]
    fn partial_coverage_interpolates_premultiplied() {
        assert_eq!(
            blend([0, 0, 0, 255], [255, 255, 255, 255], 128),
            [128, 128, 128, 255]
        );
        assert_eq!(
            blend([100, 100, 100, 255], [200, 200, 200, 255], 128),
            [150, 150, 150, 255]
        );
    }

    #[test]
    fn translucent_source_accumulates_alpha() {
        assert_eq!(
            blend([0, 0, 0, 0], [255, 255, 255, 128], 255),
            [128, 128, 128, 128]
        );
    }
}
