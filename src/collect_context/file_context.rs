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

use quote::quote;
use syn::{Expr, File, ImplItem, Item, ItemFn, ItemMod, ItemStruct, ItemTrait, ItemUse, Stmt};

#[derive(Debug)]
pub struct Application {
    file_name: String,
    function_name: String,
}

#[derive(Debug)]
pub struct UseItem {
    item: ItemUse,
}

#[derive(Debug)]
pub struct ModItem {
    item: ItemMod,
}

#[derive(Debug)]
pub struct StructItem {
    item: ItemStruct,
}

#[derive(Debug)]
pub struct FunctionItem {
    function_name: String,
    item: ItemFn,
    applications: Vec<Application>,
}

#[derive(Debug)]
pub struct TraitItem {
    item: ItemTrait,
}

#[derive(Debug)]
pub struct RsFile {
    pub file_name: String,
    pub uses: Vec<UseItem>,
    pub mods: Vec<ModItem>,
    pub structs: Vec<StructItem>,
    pub impls: Vec<ImplItem>,
    pub functions: Vec<FunctionItem>,
    pub traits: Vec<TraitItem>,
}

impl RsFile {
    pub fn new() -> Self {
        let rs_file = RsFile {
            file_name: String::new(),
            uses: Vec::new(),
            mods: Vec::new(),
            structs: Vec::new(),
            functions: Vec::new(),
            traits: Vec::new(),
        };
        rs_file
    }

    pub fn new_with_file_name(file_name: String) -> Self {
        let rs_file = RsFile {
            file_name: file_name,
            uses: Vec::new(),
            mods: Vec::new(),
            structs: Vec::new(),
            functions: Vec::new(),
            traits: Vec::new(),
        };
        rs_file
    }

    pub fn from_syn_file(file_name: String, syntax: &File) -> Self {
        let mut rs_file = RsFile::new_with_file_name(file_name);
        for item in syntax.items.clone() {
            match item {
                Item::Use(item_use) => {
                    rs_file.uses.push(UseItem { item: item_use });
                }
                Item::Mod(item_mod) => {
                    rs_file.mods.push(ModItem { item: item_mod });
                }
                Item::Struct(item_struct) => {
                    rs_file.structs.push(StructItem { item: item_struct });
                }
                Item::Fn(item_fn) => {
                    let function_name: String = item_fn.sig.ident.to_string();
                    let mut applications: Vec<Application> = Vec::new();
                    for stmt in &item_fn.block.stmts {
                        if let Stmt::Expr(expr, _) = stmt {
                            if let Expr::Call(exprcall) = expr {
                                if let Expr::Path(exprpath) = &*exprcall.func {
                                    if let Some(ident) = exprpath.path.segments.last() {
                                        let application = Application {
                                            file_name: String::from(""),
                                            function_name: ident.ident.to_string(),
                                        };
                                        applications.push(application);
                                    }
                                }
                            }
                        }
                    }
                    rs_file.functions.push(FunctionItem {
                        function_name: function_name,
                        item: item_fn,
                        applications: applications,
                    });
                }
                Item::Trait(item_trait) => {
                    rs_file.traits.push(TraitItem { item: item_trait });
                }
                _ => {}
            }
        }
        rs_file
    }

    pub fn to_string(&self) -> String {
        let uses = self.uses.iter().map(|use_item| &use_item.item);
        let mods = self.mods.iter().map(|mod_item| &mod_item.item);
        let structs = self.structs.iter().map(|struct_item| &struct_item.item);
        let functions = self
            .functions
            .iter()
            .map(|function_item| &function_item.item);
        let traits = self.traits.iter().map(|trait_item| &trait_item.item);
        let tokens = quote! {
            #(#uses)*
            #(#mods)*
            #(#structs)*
            #(#functions)*
            #(#traits)*
        };
        tokens.to_string()
    }
}
