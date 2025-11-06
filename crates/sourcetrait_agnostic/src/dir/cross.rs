#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    HomeCache,
    HomeConfig,
    HomeData,
    HomeState,
}