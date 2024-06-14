#[cfg(test)]
mod tests {
    use asmov_common_dataset_enum::{DatasetFieldEnum, Treatment, EnumTrait};

    #[test]
    fn test_dataset_field_enum() {
        #[derive(asmov_common_dataset_enum_derive::DatasetFieldEnum)]
        enum MyEnum {
            Alpha,
            Bravo,
            Charlie
        }

        assert_eq!("alpha", MyEnum::Alpha.name());
        assert_eq!("bravo", MyEnum::Bravo.name());
        assert_eq!("charlie", MyEnum::Charlie.name());

        assert_eq!(0, MyEnum::Alpha.ordinal());
        assert_eq!(1, MyEnum::Bravo.ordinal());
        assert_eq!(2, MyEnum::Charlie.ordinal());
    }
}
