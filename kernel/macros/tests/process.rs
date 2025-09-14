mod tests {
    use bern_kernel_macros::new_process;

    #[test]
    fn enum_bits() {
        new_process!(example, 4096);
    }
}
