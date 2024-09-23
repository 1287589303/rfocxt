// use syn::{Item, UseTree};

// #[derive(Default, Debug)]
// struct TypeContext {

// }

// #[derive(Default, Debug)]
// struct ModContext {
//     mod_name: String,
// }

// #[derive(Default, Debug)]
// struct UseContext {
//     is_group: bool,
//     first_use_name: String,
//     alias_name: String,
//     second_use_names: Vec<UseContext>,
// }

// impl UseContext {
//     fn recursion_construct(&mut self, use_tree: &UseTree) {
//         match use_tree {
//             UseTree::Path(path) => {
//                 self.first_use_name = path.ident.to_string();
//                 let mut second_use_context = UseContext::default();
//                 second_use_context.recursion_construct(&path.tree);
//                 self.second_use_names.push(second_use_context);
//                 self.is_group = false;
//             }
//             UseTree::Name(name) => {
//                 self.first_use_name = name.ident.to_string();
//                 self.is_group = false;
//             }
//             UseTree::Rename(rename) => {
//                 self.first_use_name = rename.ident.to_string();
//                 self.alias_name = rename.rename.to_string();
//                 self.is_group = false;
//             }
//             UseTree::Glob(_) => {
//                 self.first_use_name = String::from("*");
//                 self.is_group = false;
//             }
//             UseTree::Group(group) => {
//                 self.is_group = true;
//                 for item in &group.items {
//                     let mut second_use_context = UseContext::default();
//                     second_use_context.recursion_construct(item);
//                     self.second_use_names.push(second_use_context);
//                 }
//             }
//         }
//     }
// }

// #[derive(Default, Debug)]
// struct StructContext {
//     strcut_name: String,
//     fields: Vec<String>,
// }

// #[derive(Default, Debug)]
// pub struct FileContext {
//     file_name: String,
//     mods: Vec<ModContext>,
//     uses: Vec<UseContext>,
//     structs: Vec<StructContext>,
// }

// pub fn collect_file_context(file_name: String, syntax: &syn::File) -> FileContext {
//     let mut a_file = FileContext::default();
//     a_file.file_name = file_name;
//     for item in &syntax.items {
//         match item {
//             Item::Mod(item_mod) => {
//                 let mut a_mod = ModContext::default();
//                 a_mod.mod_name = item_mod.ident.to_string();
//                 a_file.mods.push(a_mod);
//             }
//             Item::Use(item_use) => {
//                 let mut a_use = UseContext::default();
//                 a_use.recursion_construct(&item_use.tree);
//                 a_file.uses.push(a_use);
//             }
//             Item::Struct(item_struct) => {
//                 let mut a_struct = StructContext::default();
//                 a_struct.strcut_name = item_struct.ident.to_string();
//                 a_file.structs.push(a_struct);
//             }
//             _ => {}
//         }
//     }
//     a_file
// }

use clap::builder::Str;
use quote::quote;
use syn::{
    visit::{self, Visit},
    Expr, File, Item, ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl,
    ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion,
    ItemUse, Path, Stmt, Type,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Application {
    application_name: String,
}

#[derive(Debug, Clone)]
pub struct ConstItem {
    item: Option<ItemConst>,
}

impl ConstItem {
    fn new() -> Self {
        ConstItem { item: None }
    }
}

#[derive(Debug, Clone)]
pub struct ExternCrateItem {
    item: Option<ItemExternCrate>,
}

impl ExternCrateItem {
    fn new() -> Self {
        ExternCrateItem { item: None }
    }
}

#[derive(Debug, Clone)]
pub struct ForeignModItem {
    item: Option<ItemForeignMod>,
}

impl ForeignModItem {
    fn new() -> Self {
        ForeignModItem { item: None }
    }
}

#[derive(Debug, Clone)]
pub struct MacroItem {
    item: Option<ItemMacro>,
}

impl MacroItem {
    fn new() -> Self {
        MacroItem { item: None }
    }
}

#[derive(Debug, Clone)]
pub struct TraitAliasItem {
    item: Option<ItemTraitAlias>,
}

impl TraitAliasItem {
    fn new() -> Self {
        TraitAliasItem { item: None }
    }
}

#[derive(Debug, Clone)]
pub struct UseItem {
    item: Option<ItemUse>,
}

impl UseItem {
    fn new() -> Self {
        UseItem { item: None }
    }
}

#[derive(Debug, Clone)]
pub struct ModItem {
    item: Option<ItemMod>,
}

impl ModItem {
    fn new() -> Self {
        ModItem { item: None }
    }
}

#[derive(Debug, Clone)]
pub struct StaticItem {
    item: Option<ItemStatic>,
}

impl StaticItem {
    fn new() -> Self {
        StaticItem { item: None }
    }
}

#[derive(Debug, Clone)]
pub struct TypeItem {
    item: Option<ItemType>,
}

impl TypeItem {
    fn new() -> Self {
        TypeItem { item: None }
    }
}

#[derive(Debug, Clone)]
pub struct ImplItem {
    item: Option<ItemImpl>,
}

impl ImplItem {
    fn new() -> Self {
        ImplItem { item: None }
    }
}

#[derive(Debug, Clone)]
pub struct StructItem {
    struct_name: String,
    item: Option<ItemStruct>,
    impls: Vec<ImplItem>,
    traits: Vec<String>,
    traits_impls: Vec<ImplItem>,
}

impl StructItem {
    fn new() -> Self {
        StructItem {
            struct_name: String::new(),
            item: None,
            impls: Vec::new(),
            traits: Vec::new(),
            traits_impls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnumItem {
    enum_name: String,
    item: Option<ItemEnum>,
    impls: Vec<ImplItem>,
    traits: Vec<String>,
    traits_impls: Vec<ImplItem>,
}

impl EnumItem {
    fn new() -> Self {
        EnumItem {
            enum_name: String::new(),
            item: None,
            impls: Vec::new(),
            traits: Vec::new(),
            traits_impls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnionItem {
    union_name: String,
    item: Option<ItemUnion>,
    impls: Vec<ImplItem>,
    traits: Vec<String>,
    traits_impls: Vec<ImplItem>,
}

impl UnionItem {
    fn new() -> Self {
        UnionItem {
            union_name: String::new(),
            item: None,
            impls: Vec::new(),
            traits: Vec::new(),
            traits_impls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionItem {
    function_name: String,
    item: Option<ItemFn>,
    applications: Vec<Application>,
}

impl FunctionItem {
    fn new() -> Self {
        FunctionItem {
            function_name: String::new(),
            item: None,
            applications: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraitItem {
    trait_name: String,
    item: Option<ItemTrait>,
}

impl TraitItem {
    fn new() -> Self {
        TraitItem {
            trait_name: String::new(),
            item: None,
        }
    }
}

struct PathVisitor {
    paths: Vec<String>,
}

impl<'ast> Visit<'ast> for PathVisitor {
    fn visit_path(&mut self, node: &'ast Path) {
        // let path = node.segments.last().unwrap().ident.to_string();
        let path = quote! {#node}.to_string();
        self.paths.push(path);
        visit::visit_path(self, node)
    }
}

#[derive(Debug, Clone)]
pub struct SynFile {
    pub file_name: String,
    pub consts: Vec<ConstItem>,
    pub extern_crates: Vec<ExternCrateItem>,
    pub foreign_mods: Vec<ForeignModItem>,
    pub macros: Vec<MacroItem>,
    pub trait_aliases: Vec<TraitAliasItem>,
    pub uses: Vec<UseItem>,
    pub mods: Vec<ModItem>,
    pub statics: Vec<StaticItem>,
    pub types: Vec<TypeItem>,
    pub structs: Vec<StructItem>,
    pub enums: Vec<EnumItem>,
    pub unions: Vec<UnionItem>,
    pub functions: Vec<FunctionItem>,
    pub traits: Vec<TraitItem>,
}

impl SynFile {
    pub fn new() -> Self {
        SynFile {
            file_name: String::new(),
            consts: Vec::new(),
            extern_crates: Vec::new(),
            foreign_mods: Vec::new(),
            macros: Vec::new(),
            trait_aliases: Vec::new(),
            uses: Vec::new(),
            mods: Vec::new(),
            statics: Vec::new(),
            types: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            unions: Vec::new(),
            functions: Vec::new(),
            traits: Vec::new(),
        }
    }

    pub fn new_with_file_name(file_name: String) -> Self {
        let mut syn_file = SynFile::new();
        syn_file.file_name = file_name;
        syn_file
    }

    pub fn from_syntax(file_name: String, syntax: &File) -> Self {
        let mut syn_file = SynFile::new_with_file_name(file_name);
        for item in syntax.items.clone() {
            match item {
                Item::Const(item_const) => {
                    let mut const_item = ConstItem::new();
                    const_item.item = Some(item_const);
                    syn_file.consts.push(const_item);
                }
                Item::ExternCrate(item_extern_crate) => {
                    let mut extern_crate_item = ExternCrateItem::new();
                    extern_crate_item.item = Some(item_extern_crate);
                    syn_file.extern_crates.push(extern_crate_item);
                }
                Item::ForeignMod(item_foreign_mod) => {
                    let mut foreign_mod_item = ForeignModItem::new();
                    foreign_mod_item.item = Some(item_foreign_mod);
                    syn_file.foreign_mods.push(foreign_mod_item);
                }
                Item::Macro(item_macro) => {
                    let mut macro_item = MacroItem::new();
                    macro_item.item = Some(item_macro);
                    syn_file.macros.push(macro_item);
                }
                Item::TraitAlias(item_trait_alias) => {
                    let mut trait_alias_item = TraitAliasItem::new();
                    trait_alias_item.item = Some(item_trait_alias);
                    syn_file.trait_aliases.push(trait_alias_item);
                }
                Item::Use(item_use) => {
                    let mut use_item = UseItem::new();
                    use_item.item = Some(item_use);
                    syn_file.uses.push(use_item);
                }
                Item::Mod(item_mod) => {
                    let mut mod_item = ModItem::new();
                    mod_item.item = Some(item_mod);
                    syn_file.mods.push(mod_item);
                }
                Item::Static(item_static) => {
                    let mut static_item = StaticItem::new();
                    static_item.item = Some(item_static);
                    syn_file.statics.push(static_item);
                }
                Item::Type(item_type) => {
                    let mut type_item = TypeItem::new();
                    type_item.item = Some(item_type);
                    syn_file.types.push(type_item);
                }
                Item::Struct(item_struct) => {
                    let mut struct_item = StructItem::new();
                    let struct_name = item_struct.ident.to_string();
                    struct_item.struct_name = struct_name;
                    struct_item.item = Some(item_struct);
                    syn_file.structs.push(struct_item);
                }
                Item::Enum(item_enum) => {
                    let mut enum_item = EnumItem::new();
                    let enum_name = item_enum.ident.to_string();
                    enum_item.enum_name = enum_name;
                    enum_item.item = Some(item_enum);
                    syn_file.enums.push(enum_item);
                }
                Item::Union(item_union) => {
                    let mut union_item = UnionItem::new();
                    let union_name = item_union.ident.to_string();
                    union_item.union_name = union_name;
                    union_item.item = Some(item_union);
                    syn_file.unions.push(union_item);
                }
                Item::Impl(item_impl) => {
                    let mut impl_item = ImplItem::new();
                    impl_item.item = Some(item_impl.clone());
                    let mut name = String::new();
                    let ty = *item_impl.self_ty;
                    if let Type::Path(ty_path) = ty {
                        name = ty_path.path.segments.last().unwrap().ident.to_string();
                    }
                    let mut trait_name = String::new();
                    if item_impl.trait_.clone() != None {
                        trait_name = item_impl
                            .trait_
                            .unwrap()
                            .1
                            .segments
                            .last()
                            .unwrap()
                            .ident
                            .to_string();
                    }
                    if trait_name == String::new() {
                        let mut b: bool = false;
                        for struct_item in syn_file.structs.iter_mut() {
                            if struct_item.struct_name == name {
                                struct_item.impls.push(impl_item.clone());
                                b = true;
                                break;
                            }
                        }
                        if b == false {
                            for enum_item in syn_file.enums.iter_mut() {
                                if enum_item.enum_name == name {
                                    enum_item.impls.push(impl_item.clone());
                                    b = true;
                                    break;
                                }
                            }
                        }
                        if b == false {
                            for union_item in syn_file.unions.iter_mut() {
                                if union_item.union_name == name {
                                    union_item.impls.push(impl_item.clone());
                                    break;
                                }
                            }
                        }
                    } else {
                        let mut b: bool = false;
                        for struct_item in syn_file.structs.iter_mut() {
                            if struct_item.struct_name == name {
                                struct_item.traits.push(trait_name.clone());
                                struct_item.traits_impls.push(impl_item.clone());
                                b = true;
                                break;
                            }
                        }
                        if b == false {
                            for enum_item in syn_file.enums.iter_mut() {
                                if enum_item.enum_name == name {
                                    enum_item.traits.push(trait_name.clone());
                                    enum_item.traits_impls.push(impl_item.clone());
                                    b = true;
                                    break;
                                }
                            }
                        }
                        if b == false {
                            for union_item in syn_file.unions.iter_mut() {
                                if union_item.union_name == name {
                                    union_item.traits.push(trait_name.clone());
                                    union_item.traits_impls.push(impl_item.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
                Item::Fn(item_fn) => {
                    let mut function_item = FunctionItem::new();
                    let function_name: String = item_fn.sig.ident.to_string();
                    function_item.function_name = function_name;
                    function_item.item = Some(item_fn.clone());
                    let stmts = item_fn.block.stmts.clone();
                    let mut visitor = PathVisitor { paths: Vec::new() };
                    for stmt in stmts {
                        visitor.visit_stmt(&stmt);
                    }
                    function_item
                        .applications
                        .extend(visitor.paths.iter().map(|path| Application {
                            application_name: path.clone(),
                        }));
                    function_item.applications.sort();
                    function_item.applications.dedup();
                    // println!("{:#?}", function_item.applications);
                    // for stmt in stmts {
                    //     match stmt {
                    //         Stmt::Expr(expr, _) => match expr {
                    //             Expr::Call(expr_call) => {
                    //                 let func = *expr_call.func;
                    //                 if let Expr::Path(expr_path) = func {
                    //                     let path = expr_path.path;
                    //                     let tokens = quote! {#path};
                    //                     let callee = tokens.to_string();
                    //                     let application = Application::Function {
                    //                         function_name: callee,
                    //                     };
                    //                     function_item.applications.push(application);
                    //                 }
                    //             }
                    //             _ => {}
                    //         },
                    //         _ => {}
                    //     }
                    // }
                    syn_file.functions.push(function_item);
                }
                Item::Trait(item_trait) => {
                    let mut trait_item = TraitItem::new();
                    let trait_name: String = item_trait.ident.to_string();
                    trait_item.trait_name = trait_name;
                    trait_item.item = Some(item_trait);
                    syn_file.traits.push(trait_item);
                }
                _ => {}
            }
        }
        syn_file
    }

    pub fn delete_useless_application(&mut self, all_names: &Vec<String>) {
        for function in self.functions.iter_mut() {
            function
                .applications
                .retain(|application| all_names.contains(&application.application_name));
        }
    }

    pub fn get_all_names(&self) -> Vec<String> {
        let mut all_names: Vec<String> = Vec::new();
        all_names.extend(
            self.functions
                .iter()
                .map(|function_item| function_item.function_name.clone()),
        );
        all_names.extend(
            self.structs
                .iter()
                .map(|struct_item| struct_item.struct_name.clone()),
        );
        all_names.extend(
            self.enums
                .iter()
                .map(|enum_item| enum_item.enum_name.clone()),
        );
        all_names.extend(
            self.unions
                .iter()
                .map(|union_item| union_item.union_name.clone()),
        );
        all_names
    }

    pub fn to_string(&self) -> String {
        let mut items: Vec<Item> = Vec::new();
        items.extend(
            self.types
                .iter()
                .map(|type_item| Item::Type(type_item.item.clone().unwrap())),
        );
        items.extend(
            self.uses
                .iter()
                .map(|use_item| Item::Use(use_item.item.clone().unwrap())),
        );
        items.extend(
            self.mods
                .iter()
                .map(|mod_item| Item::Mod(mod_item.item.clone().unwrap())),
        );
        items.extend(
            self.extern_crates.iter().map(|extern_crate_item| {
                Item::ExternCrate(extern_crate_item.item.clone().unwrap())
            }),
        );
        items.extend(
            self.foreign_mods
                .iter()
                .map(|foreign_mod_item| Item::ForeignMod(foreign_mod_item.item.clone().unwrap())),
        );
        items.extend(
            self.macros
                .iter()
                .map(|macro_item| Item::Macro(macro_item.item.clone().unwrap())),
        );
        items.extend(
            self.statics
                .iter()
                .map(|static_item| Item::Static(static_item.item.clone().unwrap())),
        );
        items.extend(
            self.consts
                .iter()
                .map(|const_item| Item::Const(const_item.item.clone().unwrap())),
        );
        items.extend(
            self.trait_aliases
                .iter()
                .map(|trait_alias_item| Item::TraitAlias(trait_alias_item.item.clone().unwrap())),
        );
        items.extend(
            self.traits
                .iter()
                .map(|trait_item| Item::Trait(trait_item.item.clone().unwrap())),
        );
        for struct_item in &self.structs {
            items.push(Item::Struct(struct_item.item.clone().unwrap()));
            items.extend(
                struct_item
                    .impls
                    .iter()
                    .map(|impl_item| Item::Impl(impl_item.item.clone().unwrap())),
            );
            items.extend(
                struct_item
                    .traits_impls
                    .iter()
                    .map(|trait_impl_item| Item::Impl(trait_impl_item.item.clone().unwrap())),
            );
        }
        for enum_item in &self.enums {
            items.push(Item::Enum(enum_item.item.clone().unwrap()));
            items.extend(
                enum_item
                    .impls
                    .iter()
                    .map(|impl_item| Item::Impl(impl_item.item.clone().unwrap())),
            );
            items.extend(
                enum_item
                    .traits_impls
                    .iter()
                    .map(|trait_impl_item| Item::Impl(trait_impl_item.item.clone().unwrap())),
            );
        }
        for union_item in &self.unions {
            items.push(Item::Union(union_item.item.clone().unwrap()));
            items.extend(
                union_item
                    .impls
                    .iter()
                    .map(|impl_item| Item::Impl(impl_item.item.clone().unwrap())),
            );
            items.extend(
                union_item
                    .traits_impls
                    .iter()
                    .map(|trait_impl_item| Item::Impl(trait_impl_item.item.clone().unwrap())),
            );
        }
        items.extend(
            self.functions
                .iter()
                .map(|function_item| Item::Fn(function_item.item.clone().unwrap())),
        );
        let tokens = quote! {
            #(#items)*
        };
        tokens.to_string()
    }

    pub fn get_trait(&self, trait_name: String) -> Option<ItemTrait> {
        for trait_item in self.traits.iter() {
            if trait_item.trait_name == trait_name {
                return trait_item.item.clone();
            }
        }
        None
    }

    pub fn get_items(&self, name: String) -> Vec<Item> {
        let mut items: Vec<Item> = Vec::new();
        let mut b = false;
        for function_item in self.functions.iter() {
            if function_item.function_name == name {
                items.push(Item::Fn(function_item.item.clone().unwrap()));
                b = true;
                break;
            }
        }
        if b == true {
            return items;
        }
        for struct_item in self.structs.iter() {
            if struct_item.struct_name == name {
                items.push(Item::Struct(struct_item.item.clone().unwrap()));
                items.extend(
                    struct_item
                        .impls
                        .iter()
                        .map(|impl_item| Item::Impl(impl_item.item.clone().unwrap())),
                );
            }
        }
        return items;
    }
}

#[derive(Debug, Clone)]
pub struct SynFiles {
    syn_files: Vec<SynFile>,
    all_names: Vec<String>,
}

impl SynFiles {
    pub fn new() -> Self {
        SynFiles {
            syn_files: Vec::new(),
            all_names: Vec::new(),
        }
    }

    pub fn add_syn_file(&mut self, syn_file: SynFile) {
        self.syn_files.push(syn_file);
    }

    pub fn delete_useless_applications(&mut self) {
        for syn_file in self.syn_files.iter() {
            self.all_names
                .extend(syn_file.get_all_names().iter().map(|name| name.clone()));
        }
        for syn_file in self.syn_files.iter_mut() {
            syn_file.delete_useless_application(&self.all_names);
        }
        // println!("{:#?}", self.all_names);
    }

    pub fn cout_applications(&self) {
        for syn_file in self.syn_files.iter() {
            for function in syn_file.functions.iter() {
                println!("{:#?}", function.function_name);
                println!("{:#?}", function.applications);
            }
        }
    }
}
