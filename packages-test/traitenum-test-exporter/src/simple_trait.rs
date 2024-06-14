use asmov_common_traitenum::{EnumTrait, enumtrait};

#[enumtrait]
pub trait SimpleTrait: EnumTrait {
    #[enumtrait::Str(default("spunko"))]
    fn name(&self) -> &'static str;
    fn column(&self) -> usize;

    fn default_impl(&self) -> String {
        format!("{} :: {}", self.name(), self.column())
    }
}
