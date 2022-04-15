
mod tests {
    use bern_kernel_macros::enum_map;

    enum_map!{
        Size, u8;
        S128 = 5, 128;
        S256 = 6, 256;
    }

    #[test]
    fn enum_bits() {
        assert_eq!(Size::S128.bits(), 5);
        assert_eq!(Size::S256.bits(), 6);
    }

    #[test]
    fn enum_from_value() {
        assert_eq!(size_from!(128).bits(), Size::S128.bits());
    }
}