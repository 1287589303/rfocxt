use syn::{
    ImplItemConst, ImplItemFn, ImplItemType, Item, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse,
    TraitItemConst, TraitItemFn, TraitItemType,
};

pub enum Visibility {
    PubT,
    PubS,
    PubI(MyPath),
    Pri,
}

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

    pub fn new(import_path: &String) -> Self {
        let mut my_path = MyPath::none();
        let paths = import_path.split("::").collect::<Vec<&str>>();
        my_path.name = paths[0].to_string();
        if paths.len() > 1 {
            my_path.next = Some(Box::new(MyPath::new(&paths[1..].join("::"))));
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
    pub fn none() -> Self {
        Name {
            name: String::new(),
            complete_name: String::new(),
            import_name: MyPath::none(),
        }
    }

    pub fn new(name: &String) -> Self {
        Name {
            name: name.clone(),
            complete_name: String::new(),
            import_name: MyPath::none(),
        }
    }

    pub fn insert_complete_name(&mut self, complete_name: &String) {
        self.complete_name = complete_name.clone();
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
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
pub enum FunctionItem {
    FnItem(FnItem),
    ImplFnItem(ImplFnItem),
    TraitFnItem(TraitFnItem),
}

impl FunctionItem {
    pub fn get_complete_function_name_in_file(&self) -> String {
        match self {
            FunctionItem::FnItem(fn_item) => fn_item.get_complete_function_name_in_file(),
            FunctionItem::ImplFnItem(impl_fn_item) => {
                impl_fn_item.get_complete_function_name_in_file()
            }
            FunctionItem::TraitFnItem(trait_fn_item) => {
                trait_fn_item.get_complete_function_name_in_file()
            }
        }
    }

    pub fn get_items(&self) -> Vec<Item> {
        match self {
            FunctionItem::FnItem(fn_item) => fn_item.get_items(),
            FunctionItem::ImplFnItem(impl_fn_item) => impl_fn_item.get_items(),
            FunctionItem::TraitFnItem(trait_fn_item) => trait_fn_item.get_items(),
        }
    }
}

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

    pub fn to_item(&self) -> Item {
        Item::Const(self.item.clone().unwrap())
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

    pub fn to_item(&self) -> Item {
        Item::TraitAlias(self.item.clone().unwrap())
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

    pub fn to_item(&self) -> Item {
        Item::Use(self.item.clone().unwrap())
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
        return self.inside_items.clone();
    }

    pub fn get_item(&self) -> ItemMod {
        return self.item.clone().unwrap();
    }

    pub fn has_inside(&self) -> bool {
        if self.inside_items.len() > 0 {
            return true;
        } else {
            return false;
        }
    }

    pub fn to_item(&self) -> Item {
        let mut item = self.item.clone().unwrap();
        if self.has_inside() {
            if let Some(items) = &mut item.content {
                items.1.extend(self.inside_items.clone());
            }
        }
        Item::Mod(item.clone())
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

    pub fn to_item(&self) -> Item {
        Item::Static(self.item.clone().unwrap())
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

    pub fn to_item(&self) -> Item {
        Item::Type(self.item.clone().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnItem {
    fn_name: Name,
    complete_name_in_file: String,
    item: Option<ItemFn>,
    // has_items: bool,
    inside_items: Vec<Item>,
    // application: Applications,
}

impl FnItem {
    pub fn new() -> Self {
        FnItem {
            fn_name: Name::none(),
            complete_name_in_file: String::new(),
            // complete_function_name_in_file: String::new(),
            item: None,
            inside_items: Vec::new(),
            // application: Applications::new(),
        }
    }

    pub fn insert_function_name(&mut self, fn_name: &String) {
        self.fn_name = Name::new(fn_name);
    }

    pub fn insert_complete_name_in_file(&mut self, prefix: &String) {
        if prefix.eq("") {
            self.complete_name_in_file = self.fn_name.get_name();
        } else {
            self.complete_name_in_file = prefix.clone() + "::" + &self.fn_name.get_name();
        }
    }

    pub fn insert_item(&mut self, item: &ItemFn) {
        self.item = Some(item.clone());
    }

    pub fn insert_items(&mut self, items: &Vec<Item>) {
        self.inside_items = items.clone();
    }

    pub fn has_inside(&self) -> bool {
        if self.inside_items.len() > 0 {
            return true;
        } else {
            return false;
        }
    }

    pub fn get_complete_function_name_in_file(&self) -> String {
        self.complete_name_in_file.clone()
    }

    // pub fn get_function_name(&self) -> String {
    //     return self.function_name.clone();
    // }

    pub fn get_items(&self) -> Vec<Item> {
        return self.inside_items.clone();
    }

    // pub fn get_complete_function_name_in_file(&self) -> String {
    //     return self.complete_function_name_in_file.clone();
    // }

    // pub fn insert_applications(&mut self, applications: &Vec<String>) {
    //     self.application.insert_applications(applications);
    // }

    // pub fn get_applications(&self) -> Vec<String> {
    //     return self.application.get_applications();
    // }

    // pub fn get_item(&self) -> ItemFn {
    //     if let MyItemFn::Fn(item_function) = self.item.clone().unwrap() {
    //         return item_function;
    //     } else {
    //         eprintln!("Failed to get a fn item!");
    //         process::exit(12);
    //     }
    // }

    pub fn to_item(&self) -> Item {
        Item::Fn(self.item.clone().unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct ImplTypeItem {
    item: Option<ImplItemType>,
}

impl ImplTypeItem {
    pub fn new() -> Self {
        ImplTypeItem { item: None }
    }

    pub fn insert_item(&mut self, item: &ImplItemType) {
        self.item = Some(item.clone());
    }

    pub fn get_item(&self) -> ImplItemType {
        self.item.clone().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct ImplConstItem {
    item: Option<ImplItemConst>,
}

impl ImplConstItem {
    pub fn new() -> Self {
        ImplConstItem { item: None }
    }

    pub fn insert_item(&mut self, item: &ImplItemConst) {
        self.item = Some(item.clone());
    }

    pub fn get_item(&self) -> ImplItemConst {
        self.item.clone().unwrap()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplFnItem {
    fn_name: Name,
    complete_name_in_file: String,
    item: Option<ImplItemFn>,
    // has_items: bool,
    inside_items: Vec<Item>,
}

impl ImplFnItem {
    pub fn new() -> Self {
        ImplFnItem {
            fn_name: Name::none(),
            complete_name_in_file: String::new(),
            item: None,
            inside_items: Vec::new(),
        }
    }

    pub fn insert_item(&mut self, item: &ImplItemFn) {
        self.item = Some(item.clone());
    }

    pub fn insert_fn_name(&mut self, fn_name: &String) {
        self.fn_name = Name::new(fn_name);
    }

    pub fn insert_complete_name_in_file(&mut self, prefix: &String) {
        if prefix.eq("") {
            self.complete_name_in_file = self.fn_name.get_name();
        } else {
            self.complete_name_in_file = prefix.clone() + "::" + &self.fn_name.get_name();
        }
    }

    pub fn insert_items(&mut self, items: &Vec<Item>) {
        self.inside_items = items.clone();
    }

    pub fn has_inside(&self) -> bool {
        if self.inside_items.len() > 0 {
            return true;
        } else {
            return false;
        }
    }

    pub fn get_complete_function_name_in_file(&self) -> String {
        self.complete_name_in_file.clone()
    }

    pub fn get_items(&self) -> Vec<Item> {
        return self.inside_items.clone();
    }

    pub fn get_item(&self) -> ImplItemFn {
        self.item.clone().unwrap()
    }
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
            struct_name: Name::none(),
            trait_name: None,
            item: None,
            types: Vec::new(),
            consts: Vec::new(),
            functions: Vec::new(),
            // applications: Applications::new(),
        }
    }

    pub fn insert_impl_num(&mut self, impl_num: i32) {
        self.impl_num = impl_num;
    }

    pub fn insert_struct_name(&mut self, struct_name: &String) {
        self.struct_name = Name::new(struct_name);
    }

    pub fn insert_trait_name(&mut self, trait_name: &String) {
        self.trait_name = Some(Name::new(trait_name));
    }

    pub fn insert_item(&mut self, item: &ItemImpl) {
        self.item = Some(item.clone());
    }

    pub fn insert_type(&mut self, item: &ImplTypeItem) {
        self.types.push(item.clone());
    }

    pub fn insert_const(&mut self, item: &ImplConstItem) {
        self.consts.push(item.clone());
    }

    pub fn insert_function(&mut self, item: &ImplFnItem) {
        self.functions.push(item.clone());
    }

    pub fn get_impl_num(&self) -> i32 {
        return self.impl_num;
    }

    pub fn get_function_with_inside(&self) -> Vec<ImplFnItem> {
        let mut functions_with_inside: Vec<ImplFnItem> = Vec::new();
        for impl_fn_item in self.functions.iter() {
            if impl_fn_item.has_inside() {
                functions_with_inside.push(impl_fn_item.clone());
            }
        }
        functions_with_inside
    }

    pub fn to_item(&self) -> Item {
        let mut item_impl = self.item.clone().unwrap();
        for impl_type_item in self.types.iter() {
            item_impl
                .items
                .push(syn::ImplItem::Type(impl_type_item.get_item()));
        }
        for impl_const_item in self.consts.iter() {
            item_impl
                .items
                .push(syn::ImplItem::Const(impl_const_item.get_item()));
        }
        for impl_fn_item in self.functions.iter() {
            item_impl
                .items
                .push(syn::ImplItem::Fn(impl_fn_item.get_item()));
        }
        Item::Impl(item_impl)
    }

    pub fn get_fns(&self) -> &Vec<ImplFnItem> {
        &self.functions
    }

    // pub fn insert_applications(&mut self, applications: &Vec<String>) {
    //     self.applications.insert_applications(applications);
    // }

    // pub fn get_struct_name(&self) -> String {
    //     return self.struct_name.clone();
    // }

    // pub fn get_trait_name(&self) -> Option<String> {
    //     return self.trait_name.clone();
    // }

    // pub fn get_all_applications(&self) -> Vec<String> {
    //     let mut all_applications = self.applications.get_applications();
    //     for function_item in self.functions.iter() {
    //         all_applications.extend(function_item.get_applications());
    //     }
    //     all_applications
    // }

    // pub fn get_item(&self) -> ItemImpl {
    //     return self.item.clone().unwrap();
    // }
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
            struct_name: Name::none(),
            item: None,
            // applications: Applications::new(),
        }
    }

    pub fn insert_struct_name(&mut self, struct_name: &String) {
        self.struct_name = Name::new(struct_name);
    }

    pub fn insert_item(&mut self, item: &ItemStruct) {
        self.item = Some(item.clone());
    }

    pub fn to_item(&self) -> Item {
        Item::Struct(self.item.clone().unwrap())
    }

    // pub fn insert_applications(&mut self, applications: &Vec<String>) {
    //     self.applications.insert_applications(applications);
    // }

    // pub fn get_struct_name(&self) -> String {
    //     return self.struct_name.clone();
    // }

    // pub fn get_applications(&self) -> Vec<String> {
    //     return self.applications.get_applications();
    // }

    // pub fn get_item(&self) -> ItemStruct {
    //     return self.item.clone().unwrap();
    // }
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
            enum_name: Name::none(),
            item: None,
            // applications: Applications::new(),
        }
    }

    pub fn insert_enum_name(&mut self, enum_name: &String) {
        self.enum_name = Name::new(enum_name);
    }

    pub fn insert_item(&mut self, item: &ItemEnum) {
        self.item = Some(item.clone());
    }

    pub fn to_item(&self) -> Item {
        Item::Enum(self.item.clone().unwrap())
    }

    // pub fn insert_applications(&mut self, applications: &Vec<String>) {
    //     self.applications.insert_applications(applications);
    // }

    // pub fn get_enum_name(&self) -> String {
    //     return self.enum_name.clone();
    // }

    // pub fn get_applications(&self) -> Vec<String> {
    //     return self.applications.get_applications();
    // }

    // pub fn get_item(&self) -> ItemEnum {
    //     return self.item.clone().unwrap();
    // }
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
            union_name: Name::none(),
            item: None,
            // applications: Applications::new(),
        }
    }

    pub fn insert_union_name(&mut self, union_name: &String) {
        self.union_name = Name::new(union_name);
    }

    pub fn insert_item(&mut self, item: &ItemUnion) {
        self.item = Some(item.clone());
    }

    pub fn to_item(&self) -> Item {
        Item::Union(self.item.clone().unwrap())
    }

    // pub fn insert_applications(&mut self, applications: &Vec<String>) {
    //     self.applications.insert_applications(applications);
    // }

    // pub fn get_union_name(&self) -> String {
    //     return self.union_name.clone();
    // }

    // pub fn get_applications(&self) -> Vec<String> {
    //     return self.applications.get_applications();
    // }

    // pub fn get_item(&self) -> ItemUnion {
    //     return self.item.clone().unwrap();
    // }
}

#[derive(Debug, Clone)]
pub struct TraitTypeItem {
    item: Option<TraitItemType>,
}

impl TraitTypeItem {
    pub fn new() -> Self {
        TraitTypeItem { item: None }
    }

    pub fn insert_item(&mut self, item: &TraitItemType) {
        self.item = Some(item.clone());
    }

    pub fn get_item(&self) -> TraitItemType {
        self.item.clone().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct TraitConstItem {
    item: Option<TraitItemConst>,
}

impl TraitConstItem {
    pub fn new() -> Self {
        TraitConstItem { item: None }
    }

    pub fn insert_item(&mut self, item: &TraitItemConst) {
        self.item = Some(item.clone());
    }

    pub fn get_item(&self) -> TraitItemConst {
        self.item.clone().unwrap()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitFnItem {
    fn_name: Name,
    complete_name_in_file: String,
    item: Option<TraitItemFn>,
    // has_items: bool,
    inside_items: Vec<Item>,
}

impl TraitFnItem {
    pub fn new() -> Self {
        TraitFnItem {
            fn_name: Name::none(),
            complete_name_in_file: String::new(),
            item: None,
            inside_items: Vec::new(),
        }
    }

    pub fn insert_item(&mut self, item: &TraitItemFn) {
        self.item = Some(item.clone());
    }

    pub fn insert_fn_name(&mut self, fn_name: &String) {
        self.fn_name = Name::new(fn_name);
    }

    pub fn insert_complete_name_in_file(&mut self, prefix: &String) {
        if prefix.eq("") {
            self.complete_name_in_file = self.fn_name.get_name();
        } else {
            self.complete_name_in_file = prefix.clone() + "::" + &self.fn_name.get_name();
        }
    }

    pub fn insert_items(&mut self, items: &Vec<Item>) {
        self.inside_items = items.clone();
    }

    pub fn has_inside(&self) -> bool {
        if self.inside_items.len() > 0 {
            return true;
        } else {
            return false;
        }
    }

    pub fn get_complete_function_name_in_file(&self) -> String {
        self.complete_name_in_file.clone()
    }

    pub fn get_items(&self) -> Vec<Item> {
        return self.inside_items.clone();
    }

    pub fn get_item(&self) -> TraitItemFn {
        self.item.clone().unwrap()
    }
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
            trait_name: Name::none(),
            item: None,
            types: Vec::new(),
            consts: Vec::new(),
            functions: Vec::new(),
            // applications: Applications::new(),
        }
    }

    pub fn insert_trait_name(&mut self, trait_name: &String) {
        self.trait_name = Name::new(trait_name);
    }

    pub fn insert_item(&mut self, item: &ItemTrait) {
        self.item = Some(item.clone());
    }

    pub fn insert_type(&mut self, item: &TraitTypeItem) {
        self.types.push(item.clone());
    }

    pub fn insert_const(&mut self, item: &TraitConstItem) {
        self.consts.push(item.clone());
    }

    pub fn insert_function(&mut self, item: &TraitFnItem) {
        self.functions.push(item.clone());
    }

    pub fn get_trait_name(&self) -> String {
        return self.trait_name.get_name();
    }

    pub fn get_function_with_inside(&self) -> Vec<TraitFnItem> {
        let mut functions_with_inside: Vec<TraitFnItem> = Vec::new();
        for trait_fn_item in self.functions.iter() {
            if trait_fn_item.has_inside() {
                functions_with_inside.push(trait_fn_item.clone());
            }
        }
        functions_with_inside
    }

    pub fn to_item(&self) -> Item {
        let mut item_trait = self.item.clone().unwrap();
        for trait_type_item in self.types.iter() {
            item_trait
                .items
                .push(syn::TraitItem::Type(trait_type_item.get_item()));
        }
        for trait_const_item in self.consts.iter() {
            item_trait
                .items
                .push(syn::TraitItem::Const(trait_const_item.get_item()));
        }
        for trait_fn_item in self.functions.iter() {
            item_trait
                .items
                .push(syn::TraitItem::Fn(trait_fn_item.get_item()));
        }
        Item::Trait(item_trait)
    }

    pub fn get_fns(&self) -> &Vec<TraitFnItem> {
        &self.functions
    }

    // pub fn insert_applications(&mut self, applications: &Vec<String>) {
    //     self.applications.insert_applications(applications);
    // }

    // pub fn get_all_applications(&self) -> Vec<String> {
    //     let mut all_aplications = self.applications.get_applications();
    //     for function_item in self.functions.iter() {
    //         all_aplications.extend(function_item.get_applications());
    //     }
    //     all_aplications
    // }

    // pub fn get_item(&self) -> ItemTrait {
    //     return self.item.clone().unwrap();
    // }
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
            alias_name: alias_name,
            use_tree: MyPath::new(&use_tree),
        }
    }
}
