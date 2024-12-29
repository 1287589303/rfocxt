use std::process;

use syn::{
    ImplItemConst, ImplItemFn, ImplItemType, Item, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse,
    TraitItemConst, TraitItemFn, TraitItemType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MyPath {
    name: String,
    next: Option<Box<MyPath>>,
}

impl MyPath {
    pub fn none() -> Self {
        MyPath {
            name: String::new(),
            next: None,
        }
    }

    pub fn new(import_path: String) -> Self {
        let mut my_path = MyPath::none();
        let paths = import_path.split("::").collect::<Vec<&str>>();
        my_path.name = paths[0].to_string();
        if paths.len() > 1 {
            my_path.next = Some(Box::new(MyPath::new(paths[1..].join("::"))));
        }
        my_path
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    name: String,
    complete_name: String,
    import_name: MyPath,
}

impl Name {
    pub fn new(name: String) -> Self {
        Name {
            name: name,
            complete_name: String::new(),
            import_name: MyPath::none(),
        }
    }
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct Applications {
//     applications: Vec<String>,
// }

// impl Applications {
//     pub fn new() -> Self {
//         Applications {
//             applications: Vec::new(),
//         }
//     }

//     pub fn insert_application(&mut self, application: &String) {
//         self.applications.push(application.clone());
//     }

//     pub fn insert_applications(&mut self, applications: &Vec<String>) {
//         self.applications = applications.clone()
//     }

//     pub fn get_applications(&self) -> Vec<String> {
//         return self.applications.clone();
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct ConstItem {
    item: Option<ItemConst>,
}

impl ConstItem {
    pub fn new() -> Self {
        ConstItem { item: None }
    }

    pub fn insert_item(&mut self, item: &ItemConst) {
        self.item = Some(item.clone());
    }

    pub fn get_item(&self) -> ItemConst {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitAliasItem {
    item: Option<ItemTraitAlias>,
}

impl TraitAliasItem {
    pub fn new() -> Self {
        TraitAliasItem { item: None }
    }

    pub fn insert_item(&mut self, item: &ItemTraitAlias) {
        self.item = Some(item.clone());
    }

    pub fn get_item(&self) -> ItemTraitAlias {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseItem {
    item: Option<ItemUse>,
}

impl UseItem {
    pub fn new() -> Self {
        UseItem { item: None }
    }

    pub fn insert_item(&mut self, item: &ItemUse) {
        self.item = Some(item.clone());
    }

    pub fn get_item(&self) -> ItemUse {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModItem {
    mod_name: String,
    file_name: Option<String>,
    item: Option<ItemMod>,
    // inline: bool,
    inside_items: Vec<Item>,
}

impl ModItem {
    pub fn new() -> Self {
        ModItem {
            mod_name: String::new(),
            file_name: None,
            item: None,
            inside_items: Vec::new(),
        }
    }

    pub fn insert_mod_name(&mut self, mod_name: &String) {
        self.mod_name = mod_name.clone();
    }

    pub fn insert_file_name(&mut self, file_name: &String) {
        self.file_name = Some(file_name.clone());
    }

    pub fn insert_item(&mut self, item: &ItemMod) {
        self.item = Some(item.clone());
    }

    pub fn insert_items(&mut self, items: &Vec<Item>) {
        self.inside_items = items.clone();
    }

    pub fn get_mod_name(&self) -> String {
        return self.mod_name.clone();
    }

    pub fn get_file_name(&self) -> Option<String> {
        return self.file_name.clone();
    }

    pub fn get_items(&self) -> Vec<Item> {
        return self.items.clone();
    }

    pub fn get_item(&self) -> ItemMod {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticItem {
    item: Option<ItemStatic>,
}

impl StaticItem {
    pub fn new() -> Self {
        StaticItem { item: None }
    }

    pub fn insert_item(&mut self, item: &ItemStatic) {
        self.item = Some(item.clone());
    }

    pub fn get_item(&self) -> ItemStatic {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeItem {
    item: Option<ItemType>,
}

impl TypeItem {
    pub fn new() -> Self {
        TypeItem { item: None }
    }

    pub fn insert_item(&mut self, item: &ItemType) {
        self.item = Some(item.clone());
    }

    pub fn get_item(&self) -> ItemType {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct FnItem {
    name: Name,
    item: Option<ItemFn>,
    // has_items: bool,
    inside_items: Vec<Item>,
    // application: Applications,
}

impl FunctionItem {
    pub fn new() -> Self {
        FunctionItem {
            function_name: String::new(),
            complete_function_name_in_file: String::new(),
            item: None,
            items: Vec::new(),
            application: Applications::new(),
        }
    }

    pub fn insert_function_name(&mut self, function_name: &String) {
        self.function_name = function_name.clone();
    }

    pub fn insert_complete_function_name_in_file(&mut self, prefix: &String) {
        if self.function_name.eq("") {
            eprintln!("Function name is empty!");
            process::exit(7);
        }
        if prefix.eq("") {
            self.complete_function_name_in_file = self.function_name.clone();
        } else {
            self.complete_function_name_in_file = prefix.clone() + "::" + &self.function_name;
        }
    }

    pub fn insert_item(&mut self, item: &MyItemFn) {
        self.item = Some(item.clone());
    }

    pub fn insert_items(&mut self, items: &Vec<Item>) {
        self.items = items.clone();
    }

    pub fn has_items(&self) -> bool {
        return !self.items.is_empty();
    }

    pub fn get_function_name(&self) -> String {
        return self.function_name.clone();
    }

    pub fn get_items(&self) -> Vec<Item> {
        return self.items.clone();
    }

    pub fn get_complete_function_name_in_file(&self) -> String {
        return self.complete_function_name_in_file.clone();
    }

    pub fn insert_applications(&mut self, applications: &Vec<String>) {
        self.application.insert_applications(applications);
    }

    pub fn get_applications(&self) -> Vec<String> {
        return self.application.get_applications();
    }

    pub fn get_item(&self) -> ItemFn {
        if let MyItemFn::Fn(item_function) = self.item.clone().unwrap() {
            return item_function;
        } else {
            eprintln!("Failed to get a fn item!");
            process::exit(12);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImplTypeItem {
    item: Option<ImplItemType>,
}

#[derive(Debug, Clone)]
pub struct ImplConstItem {
    item: Option<ImplItemConst>,
}

#[derive(Debug, Clone)]
pub struct ImplFnItem {
    name: Name,
    item: Option<ImplItemFn>,
    // has_items: bool,
    inside_items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub struct ImplItem {
    impl_num: i32,
    struct_name: Name,
    trait_name: Option<Name>,
    item: Option<ItemImpl>,
    types: Vec<ImplTypeItem>,
    consts: Vec<ImplConstItem>,
    functions: Vec<ImplFnItem>,
    // applications: Applications,
}

impl ImplItem {
    pub fn new() -> Self {
        ImplItem {
            impl_num: 0,
            struct_name: String::new(),
            trait_name: None,
            item: None,
            types: Vec::new(),
            consts: Vec::new(),
            functions: Vec::new(),
            applications: Applications::new(),
        }
    }

    pub fn change_impl_num(&mut self, impl_num: i32) {
        self.impl_num = impl_num;
    }

    pub fn insert_struct_name(&mut self, struct_name: &String) {
        self.struct_name = struct_name.clone();
    }

    pub fn insert_trait_name(&mut self, trait_name: &String) {
        self.trait_name = Some(trait_name.clone());
    }

    pub fn insert_item(&mut self, item: &ItemImpl) {
        self.item = Some(item.clone());
    }

    pub fn insert_type(&mut self, item: &ImplItemType) {
        self.types.push(item.clone());
    }

    pub fn insert_const(&mut self, item: &ImplItemConst) {
        self.consts.push(item.clone());
    }

    pub fn insert_function(&mut self, item: &FunctionItem) {
        self.functions.push(item.clone());
    }

    pub fn get_impl_num(&self) -> i32 {
        return self.impl_num;
    }

    pub fn get_functions(&self) -> Vec<FunctionItem> {
        return self.functions.clone();
    }

    pub fn insert_applications(&mut self, applications: &Vec<String>) {
        self.applications.insert_applications(applications);
    }

    pub fn get_struct_name(&self) -> String {
        return self.struct_name.clone();
    }

    pub fn get_trait_name(&self) -> Option<String> {
        return self.trait_name.clone();
    }

    pub fn get_all_applications(&self) -> Vec<String> {
        let mut all_applications = self.applications.get_applications();
        for function_item in self.functions.iter() {
            all_applications.extend(function_item.get_applications());
        }
        all_applications
    }

    pub fn get_item(&self) -> ItemImpl {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructItem {
    struct_name: Name,
    item: Option<ItemStruct>,
    // applications: Applications,
}

impl StructItem {
    pub fn new() -> Self {
        StructItem {
            struct_name: String::new(),
            item: None,
            applications: Applications::new(),
        }
    }

    pub fn insert_struct_name(&mut self, struct_name: &String) {
        self.struct_name = struct_name.clone();
    }

    pub fn insert_item(&mut self, item: &ItemStruct) {
        self.item = Some(item.clone());
    }

    pub fn insert_applications(&mut self, applications: &Vec<String>) {
        self.applications.insert_applications(applications);
    }

    pub fn get_struct_name(&self) -> String {
        return self.struct_name.clone();
    }

    pub fn get_applications(&self) -> Vec<String> {
        return self.applications.get_applications();
    }

    pub fn get_item(&self) -> ItemStruct {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumItem {
    enum_name: Name,
    item: Option<ItemEnum>,
    // applications: Applications,
}

impl EnumItem {
    pub fn new() -> Self {
        EnumItem {
            enum_name: String::new(),
            item: None,
            applications: Applications::new(),
        }
    }

    pub fn insert_enum_name(&mut self, enum_name: &String) {
        self.enum_name = enum_name.clone();
    }

    pub fn insert_item(&mut self, item: &ItemEnum) {
        self.item = Some(item.clone());
    }

    pub fn insert_applications(&mut self, applications: &Vec<String>) {
        self.applications.insert_applications(applications);
    }

    pub fn get_enum_name(&self) -> String {
        return self.enum_name.clone();
    }

    pub fn get_applications(&self) -> Vec<String> {
        return self.applications.get_applications();
    }

    pub fn get_item(&self) -> ItemEnum {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionItem {
    union_name: Name,
    item: Option<ItemUnion>,
    // applications: Applications,
}

impl UnionItem {
    pub fn new() -> Self {
        UnionItem {
            union_name: String::new(),
            item: None,
            applications: Applications::new(),
        }
    }

    pub fn insert_union_name(&mut self, union_name: &String) {
        self.union_name = union_name.clone();
    }

    pub fn insert_item(&mut self, item: &ItemUnion) {
        self.item = Some(item.clone());
    }

    pub fn insert_applications(&mut self, applications: &Vec<String>) {
        self.applications.insert_applications(applications);
    }

    pub fn get_union_name(&self) -> String {
        return self.union_name.clone();
    }

    pub fn get_applications(&self) -> Vec<String> {
        return self.applications.get_applications();
    }

    pub fn get_item(&self) -> ItemUnion {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct TraitTypeItem {
    item: Option<TraitItemType>,
}

#[derive(Debug, Clone)]
pub struct TraitConstItem {
    item: Option<TraitItemConst>,
}

#[derive(Debug, Clone)]
pub struct TraitFnItem {
    name: Name,
    item: Option<TraitItemFn>,
    // has_items: bool,
    inside_items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub struct TraitItem {
    trait_name: Name,
    item: Option<ItemTrait>,
    types: Vec<TraitTypeItem>,
    consts: Vec<TraitConstItem>,
    functions: Vec<TraitFnItem>,
    // applications: Applications,
}

impl TraitItem {
    pub fn new() -> Self {
        TraitItem {
            trait_name: String::new(),
            item: None,
            types: Vec::new(),
            consts: Vec::new(),
            functions: Vec::new(),
            applications: Applications::new(),
        }
    }

    pub fn insert_trait_name(&mut self, trait_name: &String) {
        self.trait_name = trait_name.clone();
    }

    pub fn insert_item(&mut self, item: &ItemTrait) {
        self.item = Some(item.clone());
    }

    pub fn insert_type(&mut self, item: &TraitItemType) {
        self.types.push(item.clone());
    }

    pub fn insert_const(&mut self, item: &TraitItemConst) {
        self.consts.push(item.clone());
    }

    pub fn insert_function(&mut self, item: &FunctionItem) {
        self.functions.push(item.clone());
    }

    pub fn get_trait_name(&self) -> String {
        return self.trait_name.clone();
    }

    pub fn get_functions(&self) -> Vec<FunctionItem> {
        return self.functions.clone();
    }

    pub fn insert_applications(&mut self, applications: &Vec<String>) {
        self.applications.insert_applications(applications);
    }

    pub fn get_all_applications(&self) -> Vec<String> {
        let mut all_aplications = self.applications.get_applications();
        for function_item in self.functions.iter() {
            all_aplications.extend(function_item.get_applications());
        }
        all_aplications
    }

    pub fn get_item(&self) -> ItemTrait {
        return self.item.clone().unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct UseTree {
    use_name: String,
    alias_name: String,
    use_tree: MyPath,
}

impl UseTree {
    pub fn new(use_name: String, use_tree: String, alias_name: String) -> Self {
        UseTree {
            use_name: use_name,
            use_tree: use_tree,
            alias_name: alias_name,
        }
    }
}
