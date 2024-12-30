use prettyplease::unparse;
use quote::quote;
use syn::{
    parse2,
    visit::{self, Visit},
    Attribute, Expr, Fields, GenericParam, Generics, Item, Lit, Meta, Path, Stmt, Type,
    TypeParamBound, UseTree as SynUseTree,
};

use super::items_context::{
    ConstItem, EnumItem, FnItem, FunctionItem, ImplConstItem, ImplFnItem, ImplItem, ImplTypeItem,
    ModItem, StaticItem, StructItem, TraitAliasItem, TraitConstItem, TraitFnItem, TraitItem,
    TraitTypeItem, TypeItem, UnionItem, UseItem, UseTree,
};

use syn::ImplItem as SynImplItem;
use syn::TraitItem as SynTraitItem;

struct AttributePathVisitor {
    is_doc: bool,
}

impl AttributePathVisitor {
    fn new() -> Self {
        AttributePathVisitor { is_doc: true }
    }
}

impl<'ast> Visit<'ast> for AttributePathVisitor {
    fn visit_path(&mut self, node: &'ast Path) {
        for segment in node.segments.iter() {
            if !segment.ident.to_string().eq("doc") {
                self.is_doc = false;
            }
        }
        if self.is_doc == false {
            return;
        } else {
            visit::visit_path(self, node);
        }
    }
}

fn is_attr_doc(attr: &Attribute) -> bool {
    let mut attribute_path_visitor = AttributePathVisitor::new();
    match &attr.meta {
        Meta::Path(path) => {
            attribute_path_visitor.visit_path(&path);
        }
        Meta::List(list) => {
            attribute_path_visitor.visit_path(&list.path);
        }
        Meta::NameValue(name_value) => {
            attribute_path_visitor.visit_path(&name_value.path);
        }
    }
    attribute_path_visitor.is_doc
}

fn delete_doc_attributes(attrs: &Vec<Attribute>) -> Vec<Attribute> {
    let mut no_doc_attrs: Vec<Attribute> = Vec::new();
    for attr in attrs.iter() {
        if !is_attr_doc(attr) {
            no_doc_attrs.push(attr.clone());
        }
    }
    no_doc_attrs
}

// struct PathVisitor {
//     paths: Vec<String>,
// }

// impl PathVisitor {
//     fn new() -> Self {
//         PathVisitor { paths: Vec::new() }
//     }
// }

// impl<'ast> Visit<'ast> for PathVisitor {
//     fn visit_path(&mut self, node: &'ast Path) {
//         self.paths.extend(
//             node.segments
//                 .iter()
//                 .map(|segment| segment.ident.to_string()),
//         );
//         visit::visit_path(self, node);
//     }
// }

// fn visit_stmts(stmts: &Vec<Stmt>, applications: &mut Vec<String>) {
//     let mut visitor = PathVisitor::new();
//     for stmt in stmts.iter() {
//         visitor.visit_stmt(stmt);
//     }
//     applications.extend(visitor.paths);
//     applications.sort();
//     applications.dedup();
// }

// fn visit_fn_arg_or_return(types: &Vec<Type>, applications: &mut Vec<String>) {
//     let mut visitor = PathVisitor::new();
//     for a_type in types.iter() {
//         visitor.visit_type(a_type);
//     }
//     applications.extend(visitor.paths);
//     applications.sort();
//     applications.dedup();
// }

// fn visit_fields(fields: &Fields, applications: &mut Vec<String>) {
//     let mut visitor = PathVisitor::new();
//     match fields {
//         Fields::Named(field_named) => {
//             for field in field_named.named.iter() {
//                 visitor.visit_type(&field.ty);
//             }
//         }
//         Fields::Unnamed(field_unnamed) => {
//             for field in field_unnamed.unnamed.iter() {
//                 visitor.visit_type(&field.ty);
//             }
//         }
//         _ => {}
//     }
//     applications.extend(visitor.paths);
//     applications.sort();
//     applications.dedup();
// }

// fn visit_generics(generics: &Generics, applications: &mut Vec<String>) {
//     let mut visitor = PathVisitor::new();
//     for genericparam in generics.params.iter() {
//         match genericparam {
//             GenericParam::Type(type_param) => {
//                 for bound in type_param.bounds.iter() {
//                     match bound {
//                         TypeParamBound::Trait(trait_bound) => {
//                             visitor.visit_path(&trait_bound.path);
//                         }
//                         _ => {}
//                     }
//                 }
//             }
//             _ => {}
//         }
//     }
//     applications.extend(visitor.paths);
//     applications.sort();
//     applications.dedup();
// }

// fn get_applications_for_fn(function: &MyItemFn, applications: &mut Vec<String>) {
//     match function {
//         MyItemFn::Fn(item_function) => {
//             let mut types: Vec<Type> = Vec::new();
//             for fn_arg in item_function.sig.inputs.iter() {
//                 match fn_arg {
//                     FnArg::Receiver(receiver) => {
//                         types.push(*receiver.ty.clone());
//                     }
//                     FnArg::Typed(typed) => {
//                         types.push(*typed.ty.clone());
//                     }
//                 }
//             }
//             if let ReturnType::Type(_, typed) = &item_function.sig.output {
//                 types.push(*typed.clone());
//             }
//             visit_fn_arg_or_return(&types, applications);
//             visit_generics(&item_function.sig.generics, applications);
//             visit_stmts(&item_function.block.stmts, applications);
//         }
//         MyItemFn::ImplFn(item_function) => {
//             let mut types: Vec<Type> = Vec::new();
//             for fn_arg in item_function.sig.inputs.iter() {
//                 match fn_arg {
//                     FnArg::Receiver(receiver) => {
//                         types.push(*receiver.ty.clone());
//                     }
//                     FnArg::Typed(typed) => {
//                         types.push(*typed.ty.clone());
//                     }
//                 }
//             }
//             if let ReturnType::Type(_, typed) = &item_function.sig.output {
//                 types.push(*typed.clone());
//             }
//             visit_fn_arg_or_return(&types, applications);
//             visit_generics(&item_function.sig.generics, applications);
//             visit_stmts(&item_function.block.stmts, applications);
//         }
//         MyItemFn::TraitFn(item_function) => {
//             let mut types: Vec<Type> = Vec::new();
//             for fn_arg in item_function.sig.inputs.iter() {
//                 match fn_arg {
//                     FnArg::Receiver(receiver) => {
//                         types.push(*receiver.ty.clone());
//                     }
//                     FnArg::Typed(typed) => {
//                         types.push(*typed.ty.clone());
//                     }
//                 }
//             }
//             if let ReturnType::Type(_, typed) = &item_function.sig.output {
//                 types.push(*typed.clone());
//             }
//             visit_generics(&item_function.sig.generics, applications);
//             visit_fn_arg_or_return(&types, applications);
//             if let Some(block) = &item_function.default {
//                 visit_stmts(&block.stmts, applications);
//             }
//         }
//     }
// }

fn expand_use_tree(tree: &SynUseTree, expand_path: String, expanded_trees: &mut Vec<UseTree>) {
    match tree {
        SynUseTree::Path(use_path) => {
            let mut path_str = String::new();
            if expand_path != String::new() {
                path_str = expand_path + "::" + &use_path.ident.to_string();
            } else {
                path_str = use_path.ident.to_string();
            }
            expand_use_tree(&use_path.tree, path_str, expanded_trees);
        }
        SynUseTree::Name(use_name) => {
            let mut path_str = String::new();
            if expand_path != String::new() {
                path_str = expand_path + "::" + &use_name.ident.to_string();
            } else {
                path_str = use_name.ident.to_string();
            }
            let use_tree = UseTree::new(use_name.ident.to_string(), path_str, String::new());
            expanded_trees.push(use_tree);
        }
        SynUseTree::Glob(_) => {
            let mut path_str = String::new();
            if expand_path != String::new() {
                path_str = expand_path + "::*";
            } else {
                path_str = "*".to_string();
            }
            let use_tree = UseTree::new("*".to_string(), path_str, String::new());
            expanded_trees.push(use_tree);
        }
        SynUseTree::Rename(use_rename) => {
            let mut path_str = String::new();
            if expand_path != String::new() {
                path_str = expand_path + "::" + &use_rename.ident.to_string();
                let use_tree = UseTree::new(
                    use_rename.ident.to_string(),
                    path_str,
                    use_rename.rename.to_string(),
                );
                expanded_trees.push(use_tree);
            } else {
                path_str = use_rename.ident.to_string();
                let use_tree = UseTree::new(
                    use_rename.ident.to_string(),
                    path_str,
                    use_rename.rename.to_string(),
                );
                expanded_trees.push(use_tree);
            }
        }
        SynUseTree::Group(use_group) => {
            for item in use_group.items.iter() {
                expand_use_tree(item, expand_path.clone(), expanded_trees);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxContext {
    consts: Vec<ConstItem>,
    trait_aliases: Vec<TraitAliasItem>,
    uses: Vec<UseItem>,
    mods: Vec<ModItem>,
    statics: Vec<StaticItem>,
    types: Vec<TypeItem>,
    structs: Vec<StructItem>,
    enums: Vec<EnumItem>,
    unions: Vec<UnionItem>,
    impls: Vec<ImplItem>,
    functions: Vec<FnItem>,
    traits: Vec<TraitItem>,
    use_trees: Vec<UseTree>,
}

impl SyntaxContext {
    pub fn new() -> Self {
        SyntaxContext {
            consts: Vec::new(),
            trait_aliases: Vec::new(),
            uses: Vec::new(),
            mods: Vec::new(),
            statics: Vec::new(),
            types: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            unions: Vec::new(),
            impls: Vec::new(),
            functions: Vec::new(),
            traits: Vec::new(),
            use_trees: Vec::new(),
        }
    }

    pub fn from_items(items: &Vec<Item>) -> Self {
        let mut syntax_context = SyntaxContext::new();
        let mut impl_num: i32 = 0;
        let mut expanded_use_trees: Vec<UseTree> = Vec::new();
        for item in items.iter() {
            match item {
                Item::Const(item_const) => {
                    let mut const_item = ConstItem::new();
                    let mut modified_item_const = item_const.clone();
                    modified_item_const.attrs = delete_doc_attributes(&modified_item_const.attrs);
                    const_item.insert_item(&modified_item_const);
                    syntax_context.consts.push(const_item);
                }
                Item::TraitAlias(item_trait_alias) => {
                    let mut trait_alias_item = TraitAliasItem::new();
                    let mut modified_item_trait_alias = item_trait_alias.clone();
                    modified_item_trait_alias.attrs =
                        delete_doc_attributes(&modified_item_trait_alias.attrs);
                    trait_alias_item.insert_item(&modified_item_trait_alias);
                    syntax_context.trait_aliases.push(trait_alias_item);
                }
                Item::Use(item_use) => {
                    let mut use_item = UseItem::new();
                    let mut modified_item_use = item_use.clone();
                    modified_item_use.attrs = delete_doc_attributes(&modified_item_use.attrs);
                    use_item.insert_item(&modified_item_use);
                    syntax_context.uses.push(use_item);
                    let mut this_expanded_paths: Vec<UseTree> = Vec::new();
                    expand_use_tree(&item_use.tree, String::new(), &mut this_expanded_paths);
                    expanded_use_trees.extend(this_expanded_paths);
                }
                Item::Mod(item_mod) => {
                    let mut mod_item = ModItem::new();
                    mod_item.insert_mod_name(&item_mod.ident.to_string());
                    let mut modified_item_mod = item_mod.clone();
                    modified_item_mod.attrs = delete_doc_attributes(&modified_item_mod.attrs);
                    if let Some(content) = &mut modified_item_mod.content {
                        mod_item.insert_items(&content.1);
                        &content.1.clear();
                    }
                    mod_item.insert_item(&modified_item_mod);
                    for attr in modified_item_mod.attrs.iter() {
                        if let Meta::NameValue(name_value) = &attr.meta {
                            if name_value
                                .path
                                .segments
                                .first()
                                .unwrap()
                                .ident
                                .to_string()
                                .eq("path")
                            {
                                if let Expr::Lit(lit) = &name_value.value {
                                    if let Lit::Str(lit_str) = &lit.lit {
                                        mod_item.insert_file_name(&lit_str.value());
                                    }
                                }
                            }
                        }
                    }
                    syntax_context.mods.push(mod_item);
                }
                Item::Static(item_static) => {
                    let mut static_item = StaticItem::new();
                    let mut modified_item_static = item_static.clone();
                    modified_item_static.attrs = delete_doc_attributes(&modified_item_static.attrs);
                    static_item.insert_item(&modified_item_static);
                    syntax_context.statics.push(static_item);
                }
                Item::Type(item_type) => {
                    let mut type_item = TypeItem::new();
                    let mut modified_item_type = item_type.clone();
                    modified_item_type.attrs = delete_doc_attributes(&modified_item_type.attrs);
                    type_item.insert_item(&modified_item_type);
                    syntax_context.types.push(type_item);
                }
                Item::Struct(item_struct) => {
                    let mut struct_item = StructItem::new();
                    struct_item.insert_struct_name(&item_struct.ident.to_string());
                    let mut modified_item_struct = item_struct.clone();
                    modified_item_struct.attrs = delete_doc_attributes(&modified_item_struct.attrs);
                    struct_item.insert_item(&modified_item_struct);
                    syntax_context.structs.push(struct_item);
                }
                Item::Enum(item_enum) => {
                    let mut enum_item = EnumItem::new();
                    enum_item.insert_enum_name(&item_enum.ident.to_string());
                    let mut modified_item_enum = item_enum.clone();
                    modified_item_enum.attrs = delete_doc_attributes(&modified_item_enum.attrs);
                    enum_item.insert_item(&modified_item_enum);
                    syntax_context.enums.push(enum_item);
                }
                Item::Union(item_union) => {
                    let mut union_item = UnionItem::new();
                    union_item.insert_union_name(&item_union.ident.to_string());
                    let mut modified_item_union = item_union.clone();
                    modified_item_union.attrs = delete_doc_attributes(&modified_item_union.attrs);
                    union_item.insert_item(&modified_item_union);
                    syntax_context.unions.push(union_item);
                }
                Item::Impl(item_impl) => {
                    let mut impl_item = ImplItem::new();
                    impl_item.insert_impl_num(impl_num);
                    impl_num += 1;
                    let mut modified_item_impl = item_impl.clone();
                    modified_item_impl.items = Vec::new();
                    modified_item_impl.attrs = delete_doc_attributes(&modified_item_impl.attrs);
                    impl_item.insert_item(&modified_item_impl);
                    let mut struct_name = String::new();
                    let ty = *item_impl.self_ty.clone();
                    if let Type::Path(ty_path) = ty {
                        struct_name = ty_path.path.segments.last().unwrap().ident.to_string();
                    }
                    impl_item.insert_struct_name(&struct_name);
                    let mut trait_name = String::new();
                    if item_impl.trait_.clone() != None {
                        trait_name = item_impl
                            .clone()
                            .trait_
                            .unwrap()
                            .1
                            .segments
                            .last()
                            .unwrap()
                            .ident
                            .to_string();
                        impl_item.insert_trait_name(&trait_name);
                    }
                    for item in item_impl.items.iter() {
                        match item {
                            SynImplItem::Const(item_const) => {
                                let mut modified_item_const = item_const.clone();
                                modified_item_const.attrs =
                                    delete_doc_attributes(&modified_item_const.attrs);
                                let mut impl_const_item = ImplConstItem::new();
                                impl_const_item.insert_item(&modified_item_const);
                                impl_item.insert_const(&impl_const_item);
                            }
                            SynImplItem::Type(item_type) => {
                                let mut modified_item_type = item_type.clone();
                                modified_item_type.attrs =
                                    delete_doc_attributes(&modified_item_type.attrs);
                                let mut impl_type_item = ImplTypeItem::new();
                                impl_type_item.insert_item(&modified_item_type);
                                impl_item.insert_type(&impl_type_item);
                            }
                            SynImplItem::Fn(item_fn) => {
                                let mut impl_fn_item = ImplFnItem::new();
                                impl_fn_item.insert_fn_name(&item_fn.sig.ident.to_string());
                                let prefix = format!("{{impl#{}}}", impl_item.get_impl_num());
                                impl_fn_item.insert_complete_name_in_file(&prefix);
                                let mut modified_item_fn = item_fn.clone();
                                modified_item_fn.attrs =
                                    delete_doc_attributes(&modified_item_fn.attrs);
                                impl_fn_item.insert_item(&modified_item_fn);
                                let mut inside_items: Vec<Item> = Vec::new();
                                for stmt in item_fn.block.stmts.iter() {
                                    if let Stmt::Item(stmt_item) = stmt {
                                        inside_items.push(stmt_item.clone());
                                    }
                                }
                                impl_fn_item.insert_items(&inside_items);
                                impl_item.insert_function(&impl_fn_item);
                            }
                            _ => {}
                        }
                    }
                    syntax_context.impls.push(impl_item);
                }
                Item::Fn(item_fn) => {
                    let mut fn_item = FnItem::new();
                    fn_item.insert_function_name(&item_fn.sig.ident.to_string());
                    fn_item.insert_complete_name_in_file(&String::new());
                    let mut modified_item_fn = item_fn.clone();
                    modified_item_fn.attrs = delete_doc_attributes(&modified_item_fn.attrs);
                    fn_item.insert_item(&modified_item_fn);
                    let mut inside_items: Vec<Item> = Vec::new();
                    for stmt in item_fn.block.stmts.iter() {
                        if let Stmt::Item(stmt_item) = stmt {
                            inside_items.push(stmt_item.clone());
                        }
                    }
                    fn_item.insert_items(&inside_items);
                    syntax_context.functions.push(fn_item);
                }
                Item::Trait(item_trait) => {
                    let mut trait_item = TraitItem::new();
                    trait_item.insert_trait_name(&item_trait.ident.to_string());
                    let mut modified_item_trait = item_trait.clone();
                    modified_item_trait.attrs = delete_doc_attributes(&modified_item_trait.attrs);
                    modified_item_trait.items = Vec::new();
                    trait_item.insert_item(&modified_item_trait);
                    for item in item_trait.items.iter() {
                        match item {
                            SynTraitItem::Const(item_const) => {
                                let mut modified_item_const = item_const.clone();
                                modified_item_const.attrs =
                                    delete_doc_attributes(&modified_item_const.attrs);
                                let mut trait_const_item = TraitConstItem::new();
                                trait_const_item.insert_item(&modified_item_const);
                                trait_item.insert_const(&trait_const_item);
                            }
                            SynTraitItem::Type(item_type) => {
                                let mut modified_item_type = item_type.clone();
                                modified_item_type.attrs =
                                    delete_doc_attributes(&modified_item_type.attrs);
                                let mut trait_type_item = TraitTypeItem::new();
                                trait_type_item.insert_item(&modified_item_type);
                                trait_item.insert_type(&trait_type_item);
                            }
                            SynTraitItem::Fn(item_fn) => {
                                if let Some(block) = &item_fn.default {
                                    let mut trait_fn_item = TraitFnItem::new();
                                    trait_fn_item.insert_fn_name(&item_fn.sig.ident.to_string());
                                    trait_fn_item
                                        .insert_complete_name_in_file(&trait_item.get_trait_name());
                                    let mut modified_item_fn = item_fn.clone();
                                    modified_item_fn.attrs =
                                        delete_doc_attributes(&modified_item_fn.attrs);
                                    trait_fn_item.insert_item(&modified_item_fn);
                                    let mut inside_items: Vec<Item> = Vec::new();
                                    for stmt in block.stmts.iter() {
                                        if let Stmt::Item(stmt_item) = stmt {
                                            inside_items.push(stmt_item.clone());
                                        }
                                    }
                                    trait_fn_item.insert_items(&inside_items);
                                    trait_item.insert_function(&trait_fn_item);
                                }
                            }
                            _ => {}
                        }
                    }
                    trait_item.insert_item(&modified_item_trait);
                    syntax_context.traits.push(trait_item);
                }
                _ => {}
            }
        }
        syntax_context.use_trees = expanded_use_trees;
        syntax_context
    }

    pub fn get_inline_mods(&self) -> Vec<ModItem> {
        let mut inline_mods: Vec<ModItem> = Vec::new();
        for mod_item in self.mods.iter() {
            if mod_item.has_inside() {
                inline_mods.push(mod_item.clone());
            }
        }
        inline_mods
    }

    pub fn get_no_inline_mods(&self) -> Vec<ModItem> {
        let mut no_inline_mods: Vec<ModItem> = Vec::new();
        for mod_item in self.mods.iter() {
            if !mod_item.has_inside() {
                no_inline_mods.push(mod_item.clone());
            }
        }
        no_inline_mods
    }

    pub fn get_functions_with_items(&self) -> Vec<FunctionItem> {
        let mut functions: Vec<FunctionItem> = Vec::new();
        for function_item in self.functions.iter() {
            if function_item.has_inside() {
                functions.push(FunctionItem::FnItem(function_item.clone()));
            }
        }
        for impl_item in self.impls.iter() {
            functions.extend(
                impl_item
                    .get_function_with_inside()
                    .into_iter()
                    .map(|impl_fn_item| FunctionItem::ImplFnItem(impl_fn_item)),
            );
        }
        for trait_item in self.traits.iter() {
            functions.extend(
                trait_item
                    .get_function_with_inside()
                    .into_iter()
                    .map(|trait_fn_item| FunctionItem::TraitFnItem(trait_fn_item)),
            );
        }
        functions
    }

    pub fn get_all_in_file_function_names(&self) -> Vec<String> {
        let mut all_in_file_function_names: Vec<String> = Vec::new();
        for function_item in self.functions.iter() {
            all_in_file_function_names.push(function_item.get_complete_function_name_in_file());
        }
        for impl_item in self.impls.iter() {
            for function_item in impl_item.get_fns().iter() {
                all_in_file_function_names.push(function_item.get_complete_function_name_in_file());
            }
        }
        for trait_item in self.traits.iter() {
            for function_item in trait_item.get_fns().iter() {
                all_in_file_function_names.push(function_item.get_complete_function_name_in_file());
            }
        }
        all_in_file_function_names
    }

    // pub fn get_item(&self, item_name: &String) -> SyntaxContext {
    //     let mut syntax_context = SyntaxContext::new();
    //     for struct_item in self.structs.iter() {
    //         if struct_item.get_struct_name().eq(item_name) {
    //             syntax_context.structs.push(struct_item.clone());
    //         }
    //     }
    //     for enum_item in self.enums.iter() {
    //         if enum_item.get_enum_name().eq(item_name) {
    //             syntax_context.enums.push(enum_item.clone());
    //         }
    //     }
    //     for union_item in self.unions.iter() {
    //         if union_item.get_union_name().eq(item_name) {
    //             syntax_context.unions.push(union_item.clone());
    //         }
    //     }
    //     for impl_item in self.impls.iter() {
    //         let struct_name = impl_item.get_struct_name();
    //         let trait_name = impl_item.get_trait_name();
    //         if let None = trait_name {
    //             if struct_name.eq(item_name) {
    //                 syntax_context.impls.push(impl_item.clone());
    //             }
    //         } else {
    //             if struct_name.eq(item_name) || trait_name.unwrap().eq(item_name) {
    //                 syntax_context.impls.push(impl_item.clone());
    //             }
    //         }
    //     }
    //     for function_item in self.functions.iter() {
    //         if function_item.get_function_name().eq(item_name) {
    //             syntax_context.functions.push(function_item.clone());
    //         }
    //     }
    //     for trait_item in self.traits.iter() {
    //         if trait_item.get_trait_name().eq(item_name) {
    //             syntax_context.traits.push(trait_item.clone());
    //         }
    //     }
    //     syntax_context
    // }

    // pub fn get_simplified_item(&self, item_name: &String) -> SyntaxContext {
    //     let mut syntax_context = SyntaxContext::new();
    //     for struct_item in self.structs.iter() {
    //         if struct_item.get_struct_name().eq(item_name) {
    //             syntax_context.structs.push(struct_item.clone());
    //         }
    //     }
    //     for enum_item in self.enums.iter() {
    //         if enum_item.get_enum_name().eq(item_name) {
    //             syntax_context.enums.push(enum_item.clone());
    //         }
    //     }
    //     for union_item in self.unions.iter() {
    //         if union_item.get_union_name().eq(item_name) {
    //             syntax_context.unions.push(union_item.clone());
    //         }
    //     }
    //     for impl_item in self.impls.iter() {
    //         let struct_name = impl_item.get_struct_name();
    //         let trait_name = impl_item.get_trait_name();
    //         if let None = trait_name {
    //             if struct_name.eq(item_name) {
    //                 let mut simplified_impl_item = impl_item.clone();
    //                 simplified_impl_item.clear_stmts();
    //                 syntax_context.impls.push(simplified_impl_item);
    //             }
    //         } else {
    //             if struct_name.eq(item_name) || trait_name.unwrap().eq(item_name) {
    //                 let mut simplified_impl_item = impl_item.clone();
    //                 simplified_impl_item.clear_stmts();
    //                 syntax_context.impls.push(simplified_impl_item);
    //             }
    //         }
    //     }
    //     for function_item in self.functions.iter() {
    //         if function_item.get_function_name().eq(item_name) {
    //             let mut simplified_function_item = function_item.clone();
    //             simplified_function_item.clear_stmts();
    //             syntax_context.functions.push(simplified_function_item);
    //         }
    //     }
    //     for trait_item in self.traits.iter() {
    //         if trait_item.get_trait_name().eq(item_name) {
    //             let mut simplified_trait_item = trait_item.clone();
    //             simplified_trait_item.clear_stmts();
    //             syntax_context.traits.push(simplified_trait_item);
    //         }
    //     }
    //     syntax_context
    // }

    // pub fn get_all_applications(&self) -> Vec<String> {
    //     let mut all_applications: Vec<String> = Vec::new();
    //     for struct_item in self.structs.iter() {
    //         all_applications.extend(struct_item.get_applications());
    //     }
    //     for enum_item in self.enums.iter() {
    //         all_applications.extend(enum_item.get_applications());
    //     }
    //     for union_item in self.unions.iter() {
    //         all_applications.extend(union_item.get_applications());
    //     }
    //     for impl_item in self.impls.iter() {
    //         all_applications.extend(impl_item.get_all_applications());
    //     }
    //     for function_item in self.functions.iter() {
    //         all_applications.extend(function_item.get_applications());
    //     }
    //     for trait_item in self.traits.iter() {
    //         all_applications.extend(trait_item.get_all_applications());
    //     }
    //     all_applications.sort();
    //     all_applications.dedup();
    //     all_applications
    // }

    // pub fn extend_with_other(&mut self, syntax_context: &SyntaxContext) {
    //     for const_item in syntax_context.consts.iter() {
    //         if !self.consts.contains(&const_item) {
    //             self.consts.push(const_item.clone());
    //         }
    //     }
    //     for trait_alias_item in syntax_context.trait_aliases.iter() {
    //         if !self.trait_aliases.contains(&trait_alias_item) {
    //             self.trait_aliases.push(trait_alias_item.clone());
    //         }
    //     }
    //     for use_item in syntax_context.uses.iter() {
    //         if !self.uses.contains(&use_item) {
    //             self.uses.push(use_item.clone());
    //         }
    //     }
    //     for mod_item in syntax_context.mods.iter() {
    //         if !self.mods.contains(&mod_item) {
    //             self.mods.push(mod_item.clone());
    //         }
    //     }
    //     for static_item in syntax_context.statics.iter() {
    //         if !self.statics.contains(&static_item) {
    //             self.statics.push(static_item.clone());
    //         }
    //     }
    //     for type_item in syntax_context.types.iter() {
    //         if !self.types.contains(&type_item) {
    //             self.types.push(type_item.clone());
    //         }
    //     }
    //     for struct_item in syntax_context.structs.iter() {
    //         if !self.structs.contains(&struct_item) {
    //             self.structs.push(struct_item.clone());
    //         }
    //     }
    //     for enum_item in syntax_context.enums.iter() {
    //         if !self.enums.contains(&enum_item) {
    //             self.enums.push(enum_item.clone());
    //         }
    //     }
    //     for union_item in syntax_context.unions.iter() {
    //         if !self.unions.contains(&union_item) {
    //             self.unions.push(union_item.clone());
    //         }
    //     }
    //     for impl_item in syntax_context.impls.iter() {
    //         if !self.impls.contains(&impl_item) {
    //             self.impls.push(impl_item.clone());
    //         }
    //     }
    //     for function_item in syntax_context.functions.iter() {
    //         if !self.functions.contains(&function_item) {
    //             self.functions.push(function_item.clone());
    //         }
    //     }
    //     for trait_item in syntax_context.traits.iter() {
    //         if !self.traits.contains(&trait_item) {
    //             self.traits.push(trait_item.clone());
    //         }
    //     }
    // }

    // fn get_impl_structs(&self) -> Vec<String> {
    //     let mut structs: Vec<String> = Vec::new();
    //     for impl_item in self.impls.iter() {
    //         if let Some(_) = impl_item.get_trait_name() {
    //             structs.push(impl_item.get_struct_name());
    //         }
    //     }
    //     structs
    // }

    // fn get_impl_traits(&self) -> Vec<String> {
    //     let mut traits: Vec<String> = Vec::new();
    //     for impl_item in self.impls.iter() {
    //         if let Some(trait_name) = impl_item.get_trait_name() {
    //             traits.push(trait_name);
    //         }
    //     }
    //     traits
    // }

    // pub fn get_context(
    //     &self,
    //     output_path: &PathBuf,
    //     mod_tree: &String,
    //     main_mod_contexts: &Vec<ModContext>,
    // ) {
    //     for function_item in self.functions.iter() {
    //         let complete_function_name =
    //             mod_tree.clone() + "::" + &function_item.get_complete_function_name_in_file();
    //         let mut remain_applications: Vec<String> = Vec::new();
    //         let mut already_applications: Vec<String> = Vec::new();
    //         let mut syntax_context = SyntaxContext::new();
    //         syntax_context.functions.push(function_item.clone());
    //         already_applications.push(function_item.get_function_name());
    //         remain_applications.extend(function_item.get_applications());
    //         while !remain_applications.is_empty() {
    //             let item_name = remain_applications.remove(0);
    //             if !already_applications.contains(&item_name) {
    //                 already_applications.push(item_name.clone());
    //                 let mut named_syntax_context = SyntaxContext::new();
    //                 for main_mod_context in main_mod_contexts.iter() {
    //                     main_mod_context.get_all_item(&item_name, &mut named_syntax_context);
    //                 }
    //                 syntax_context.extend_with_other(&named_syntax_context);
    //             }
    //         }
    //         let traits = syntax_context.get_impl_traits();
    //         for trait_name in traits.iter() {
    //             if !already_applications.contains(trait_name) {
    //                 remain_applications.push(trait_name.clone());
    //             }
    //         }
    //         while !remain_applications.is_empty() {
    //             let item_name = remain_applications.remove(0);
    //             already_applications.push(item_name.clone());
    //             let mut named_syntax_context = SyntaxContext::new();
    //             for main_mod_context in main_mod_contexts.iter() {
    //                 main_mod_context.get_all_item(&item_name, &mut named_syntax_context);
    //             }
    //             syntax_context.extend_with_other(&named_syntax_context);
    //         }
    //         // let structs = syntax_context.get_impl_structs();
    //         // for struct_name in structs.iter() {
    //         //     if !already_applications.contains(struct_name) {
    //         //         remain_applications.push(struct_name.clone());
    //         //     }
    //         // }
    //         // while !remain_applications.is_empty() {
    //         //     let item_name = remain_applications.remove(0);
    //         //     already_applications.push(item_name.clone());
    //         //     let mut named_syntax_context = SyntaxContext::new();
    //         //     for main_mod_context in main_mod_contexts.iter() {
    //         //         main_mod_context.get_all_item(&item_name, &mut named_syntax_context);
    //         //     }
    //         //     syntax_context.extend_with_other(&named_syntax_context);
    //         // }
    //         let rs_file_name = complete_function_name + ".rs";
    //         let output_file_path = output_path.join(rs_file_name);
    //         let mut file = File::create(output_file_path).unwrap();
    //         file.write_all(syntax_context.to_string().as_bytes())
    //             .unwrap();
    //     }
    //     for impl_item in self.impls.iter() {
    //         for function_item in impl_item.get_functions().iter() {
    //             let complete_function_name =
    //                 mod_tree.clone() + "::" + &function_item.get_complete_function_name_in_file();
    //             let mut remain_applications: Vec<String> = Vec::new();
    //             let mut already_applications: Vec<String> = Vec::new();
    //             let struct_name = impl_item.get_struct_name();
    //             remain_applications.push(struct_name);
    //             let trait_name = impl_item.get_trait_name();
    //             if let Some(trait_name) = trait_name {
    //                 remain_applications.push(trait_name);
    //             }
    //             remain_applications.extend(function_item.get_applications());
    //             let mut syntax_context = SyntaxContext::new();
    //             while !remain_applications.is_empty() {
    //                 let item_name = remain_applications.remove(0);
    //                 if !already_applications.contains(&item_name) {
    //                     already_applications.push(item_name.clone());
    //                     let mut named_syntax_context = SyntaxContext::new();
    //                     for main_mod_context in main_mod_contexts.iter() {
    //                         main_mod_context.get_all_item(&item_name, &mut named_syntax_context);
    //                     }
    //                     syntax_context.extend_with_other(&named_syntax_context);
    //                 }
    //             }
    //             let traits = syntax_context.get_impl_traits();
    //             for trait_name in traits.iter() {
    //                 if !already_applications.contains(trait_name) {
    //                     remain_applications.push(trait_name.clone());
    //                 }
    //             }
    //             while !remain_applications.is_empty() {
    //                 let item_name = remain_applications.remove(0);
    //                 already_applications.push(item_name.clone());
    //                 let mut named_syntax_context = SyntaxContext::new();
    //                 for main_mod_context in main_mod_contexts.iter() {
    //                     main_mod_context.get_all_item(&item_name, &mut named_syntax_context);
    //                 }
    //                 syntax_context.extend_with_other(&named_syntax_context);
    //             }
    //             // let structs = syntax_context.get_impl_structs();
    //             // for struct_name in structs.iter() {
    //             //     if !already_applications.contains(struct_name) {
    //             //         remain_applications.push(struct_name.clone());
    //             //     }
    //             // }
    //             // while !remain_applications.is_empty() {
    //             //     let item_name = remain_applications.remove(0);
    //             //     already_applications.push(item_name.clone());
    //             //     let mut named_syntax_context = SyntaxContext::new();
    //             //     for main_mod_context in main_mod_contexts.iter() {
    //             //         main_mod_context.get_all_item(&item_name, &mut named_syntax_context);
    //             //     }
    //             //     syntax_context.extend_with_other(&named_syntax_context);
    //             // }
    //             let rs_file_name = complete_function_name + ".rs";
    //             let output_file_path = output_path.join(rs_file_name);
    //             let mut file = File::create(output_file_path).unwrap();
    //             file.write_all(syntax_context.to_string().as_bytes())
    //                 .unwrap();
    //         }
    //     }
    //     for trait_item in self.traits.iter() {
    //         for function_item in trait_item.get_functions().iter() {
    //             let complete_function_name =
    //                 mod_tree.clone() + "::" + &function_item.get_complete_function_name_in_file();
    //             let mut remain_applications: Vec<String> = Vec::new();
    //             let mut already_applications: Vec<String> = Vec::new();
    //             let trait_name = trait_item.get_trait_name();
    //             remain_applications.push(trait_name);
    //             let mut syntax_context = SyntaxContext::new();
    //             remain_applications.extend(function_item.get_applications());
    //             while !remain_applications.is_empty() {
    //                 let item_name = remain_applications.remove(0);
    //                 if !already_applications.contains(&item_name) {
    //                     already_applications.push(item_name.clone());
    //                     let mut named_syntax_context = SyntaxContext::new();
    //                     for main_mod_context in main_mod_contexts.iter() {
    //                         main_mod_context.get_all_item(&item_name, &mut named_syntax_context);
    //                     }
    //                     syntax_context.extend_with_other(&named_syntax_context);
    //                 }
    //             }
    //             let traits = syntax_context.get_impl_traits();
    //             for trait_name in traits.iter() {
    //                 if !already_applications.contains(trait_name) {
    //                     remain_applications.push(trait_name.clone());
    //                 }
    //             }
    //             while !remain_applications.is_empty() {
    //                 let item_name = remain_applications.remove(0);
    //                 already_applications.push(item_name.clone());
    //                 let mut named_syntax_context = SyntaxContext::new();
    //                 for main_mod_context in main_mod_contexts.iter() {
    //                     main_mod_context.get_all_item(&item_name, &mut named_syntax_context);
    //                 }
    //                 syntax_context.extend_with_other(&named_syntax_context);
    //             }
    //             // let structs = syntax_context.get_impl_structs();
    //             // for struct_name in structs.iter() {
    //             //     if !already_applications.contains(struct_name) {
    //             //         remain_applications.push(struct_name.clone());
    //             //     }
    //             // }
    //             // while !remain_applications.is_empty() {
    //             //     let item_name = remain_applications.remove(0);
    //             //     already_applications.push(item_name.clone());
    //             //     let mut named_syntax_context = SyntaxContext::new();
    //             //     for main_mod_context in main_mod_contexts.iter() {
    //             //         main_mod_context.get_all_item(&item_name, &mut named_syntax_context);
    //             //     }
    //             //     syntax_context.extend_with_other(&named_syntax_context);
    //             // }
    //             let rs_file_name = complete_function_name + ".rs";
    //             let output_file_path = output_path.join(rs_file_name);
    //             let mut file = File::create(output_file_path).unwrap();
    //             file.write_all(syntax_context.to_string().as_bytes())
    //                 .unwrap();
    //         }
    //     }
    // }

    fn to_string(&self) -> String {
        let mut items: Vec<Item> = Vec::new();
        items.extend(self.types.iter().map(|type_item| type_item.to_item()));
        items.extend(self.uses.iter().map(|use_item| use_item.to_item()));
        items.extend(self.mods.iter().map(|mod_item| mod_item.to_item()));
        items.extend(self.statics.iter().map(|static_item| static_item.to_item()));
        items.extend(self.consts.iter().map(|const_item| const_item.to_item()));
        items.extend(
            self.trait_aliases
                .iter()
                .map(|trait_alias_item| trait_alias_item.to_item()),
        );
        items.extend(self.traits.iter().map(|trait_item| trait_item.to_item()));
        items.extend(self.structs.iter().map(|struct_item| struct_item.to_item()));
        items.extend(self.enums.iter().map(|enum_item| enum_item.to_item()));
        items.extend(self.unions.iter().map(|union_item| union_item.to_item()));
        items.extend(self.impls.iter().map(|impl_item| impl_item.to_item()));
        items.extend(
            self.functions
                .iter()
                .map(|function_item| function_item.to_item()),
        );
        let tokens = quote! {#(#items)*};
        let syntax: syn::File = parse2(tokens).unwrap();
        unparse(&syntax)
        // tokens.to_string()
    }
}
