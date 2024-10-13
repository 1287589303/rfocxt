#[derive(Debug, Clone)]
pub struct ConstItem {
    item: Option<ItemConst>,
}
impl ConstItem {
    fn new() -> Self {
        ConstItem { item: None }
    }
}
