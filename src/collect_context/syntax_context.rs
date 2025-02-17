use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::PathBuf,
    rc::Rc,
};

use call_chain::analysis::exporter::CallsAndTypes;
use prettyplease::unparse;
use quote::quote;
use regex::Regex;
use syn::{
    parse2,
    visit::{self, Visit},
    Attribute, Expr, Fields, FieldsNamed, Item, Lit, Meta, Path, Stmt, Type, UseTree as SynUseTree,
    Visibility,
};

use super::{
    crate_context::CrateContext,
    items_context::{
        ConstItem, EnumItem, FnItem, FunctionItem, ImplConstItem, ImplFnItem, ImplItem,
        ImplTypeItem, ModItem, MyPath, MyVisibility, Name, StaticItem, StructItem, TraitAliasItem,
        TraitConstItem, TraitFnItem, TraitItem, TraitTypeItem, TypeItem, UnionItem, UseItem,
        UseTree,
    },
    mod_context::ModContext,
    result::{FnData, FnType, StructData, StructType},
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

fn parse_visibility(visibility: &Visibility) -> MyVisibility {
    match visibility {
        Visibility::Public(_) => MyVisibility::PubT,
        Visibility::Restricted(restricted) => {
            if let Some(_) = restricted.in_token {
                let path = &restricted.path;
                let mut paths: Vec<String> = Vec::new();
                for p in path.segments.iter() {
                    paths.push(p.ident.to_string());
                }
                MyVisibility::PubI(MyPath::new(&paths.join("::")))
            } else {
                MyVisibility::PubS
            }
        }
        Visibility::Inherited => MyVisibility::Pri,
    }
}

struct PathVisitor {
    paths: Vec<String>,
}

impl PathVisitor {
    fn new() -> Self {
        PathVisitor { paths: Vec::new() }
    }
}

impl<'ast> Visit<'ast> for PathVisitor {
    fn visit_path(&mut self, node: &'ast Path) {
        self.paths.extend(
            node.segments
                .iter()
                .map(|segment| segment.ident.to_string()),
        );
        visit::visit_path(self, node);
    }
}

fn visit_fields(fields: &Fields, applications: &mut Vec<String>) {
    let mut visitor = PathVisitor::new();
    match fields {
        Fields::Named(field_named) => {
            for field in field_named.named.iter() {
                visitor.visit_type(&field.ty);
            }
        }
        Fields::Unnamed(field_unnamed) => {
            for field in field_unnamed.unnamed.iter() {
                visitor.visit_type(&field.ty);
            }
        }
        _ => {}
    }
    applications.extend(visitor.paths);
    applications.sort();
    applications.dedup();
}

fn visit_fields_named(fields_named: &FieldsNamed, applications: &mut Vec<String>) {
    let mut visitor = PathVisitor::new();
    for field in fields_named.named.iter() {
        visitor.visit_type(&field.ty);
    }
    applications.extend(visitor.paths);
    applications.sort();
    applications.dedup();
}

fn add_new_calls_and_types(data: &mut CallsAndTypes, mod_trees: &Vec<String>) {
    let re_impl = Regex::new(r"<impl\s([^>]+)>").unwrap();
    let re_as = Regex::new(r"<([^>\s]+)\sas\s([^>\s]+)>").unwrap();
    let re_trait_bound = Regex::new(r"(::<[^>\s]+[,\s[^>\s]+]*>)").unwrap();
    let re_struct = Regex::new(r"(<[^>\s]+[,\s[^>\s]+]*>)").unwrap();
    let mut new_calls: HashSet<String> = HashSet::new();
    let mut new_types: HashSet<String> = HashSet::new();
    for call in data.calls.iter() {
        for caps in re_impl.captures_iter(&call) {
            let content = caps[1].to_string();
            let path = MyPath::new(&content);
            for mod_tree in mod_trees.iter() {
                let mod_tree_path = MyPath::new(mod_tree);
                let new_path = mod_tree_path.connect(&path).to_string();
                let new_call = call.replace(&content, &new_path);
                new_calls.insert(new_call.to_string());
            }
        }
        for caps in re_as.captures_iter(&call) {
            let content1 = caps[1].to_string();
            let content2 = caps[2].to_string();

            let new_call = call.replace(&content2, "");
            new_calls.insert(new_call);
            let path = MyPath::new(&content1);
            for mod_tree in mod_trees.iter() {
                let mod_tree_path = MyPath::new(mod_tree);
                let new_path = mod_tree_path.connect(&path).to_string();
                let new_call = call.replace(&content1, &new_path);
                new_calls.insert(new_call.to_string());

                let new_call = new_call.replace(&content2, "");
                new_calls.insert(new_call);
            }

            let new_call = call.replace(&content1, " ");
            new_calls.insert(new_call.to_string());

            let path = MyPath::new(&content2);
            for mod_tree in mod_trees.iter() {
                let mod_tree_path = MyPath::new(mod_tree);
                let new_path = mod_tree_path.connect(&path).to_string();
                let new_call = call.replace(&content2, &new_path);
                new_calls.insert(new_call.to_string());

                let new_call = new_call.replace(&content1, " ");
                new_calls.insert(new_call);
            }
        }
        for caps in re_trait_bound.captures_iter(&call) {
            let content = caps[1].to_string();
            // println!("{}", content);
            let new_call = call.replace(&content, &"");
            new_calls.insert(new_call);
        }
    }
    for new_call in new_calls {
        if !data.calls.contains(&new_call) {
            data.calls.push(new_call);
        }
    }
    new_calls = HashSet::new();
    for call in data.calls.iter() {
        for mod_tree in mod_trees.iter() {
            let mod_tree_path = MyPath::new(mod_tree);
            let call_path = MyPath::new(call);
            let new_call = mod_tree_path.connect(&call_path);
            new_calls.insert(new_call.to_string());
        }
    }
    for a_type in data.types.iter() {
        for caps in re_impl.captures_iter(&a_type) {
            let content = caps[1].to_string();
            let path = MyPath::new(&content);
            for mod_tree in mod_trees.iter() {
                let mod_tree_path = MyPath::new(mod_tree);
                let new_path = mod_tree_path.connect(&path).to_string();
                let new_type = a_type.replace(&content, &new_path);
                new_types.insert(new_type.to_string());
            }
        }
        for caps in re_as.captures_iter(&a_type) {
            let content1 = caps[1].to_string();
            let content2 = caps[2].to_string();

            let new_type = a_type.replace(&content2, "");
            new_types.insert(new_type);
            let path = MyPath::new(&content1);
            for mod_tree in mod_trees.iter() {
                let mod_tree_path = MyPath::new(mod_tree);
                let new_path = mod_tree_path.connect(&path).to_string();
                let new_type = a_type.replace(&content1, &new_path);
                new_types.insert(new_type.to_string());

                let new_type = new_type.replace(&content2, "");
                new_types.insert(new_type);
            }

            let new_type = a_type.replace(&content1, " ");
            new_types.insert(new_type.to_string());

            let path = MyPath::new(&content2);
            for mod_tree in mod_trees.iter() {
                let mod_tree_path = MyPath::new(mod_tree);
                let new_path = mod_tree_path.connect(&path).to_string();
                let new_type = a_type.replace(&content2, &new_path);
                new_types.insert(new_type.to_string());

                let new_type = new_type.replace(&content1, " ");
                new_types.insert(new_type);
            }
        }
        for caps in re_trait_bound.captures_iter(&a_type) {
            let content = caps[1].to_string();
            // println!("{}", content);
            let new_type = a_type.replace(&content, &"");
            new_types.insert(new_type);
        }
        for caps in re_struct.captures_iter(&a_type) {
            let content = caps[1].to_string();
            let new_type = a_type.replace(&content, "");
            new_types.insert(new_type);
        }
    }
    for new_type in new_types {
        if !data.types.contains(&new_type) {
            data.types.push(new_type);
        }
    }
    new_types = HashSet::new();
    for a_type in data.types.iter() {
        for mod_tree in mod_trees.iter() {
            let mod_tree_path = MyPath::new(mod_tree);
            let type_path = MyPath::new(a_type);
            let new_type = mod_tree_path.connect(&type_path);
            new_types.insert(new_type.to_string());
        }
    }
    for new_call in new_calls {
        if !data.calls.contains(&new_call) {
            data.calls.push(new_call);
        }
    }
    for new_type in new_types {
        if !data.types.contains(&new_type) {
            data.types.push(new_type);
        }
    }
}

fn get_syntax(
    data: &CallsAndTypes,
    syntax_context: &mut SyntaxContext,
    fns: &HashMap<String, FnData>,
    structs: &HashMap<String, StructData>,
) {
    for call in data.calls.iter() {
        let fn_data = fns.get(call);
        if let Some(fn_data) = fn_data {
            match &fn_data.fn_type {
                FnType::Fn(fn_item) => {
                    if !syntax_context.functions.contains(&fn_item) {
                        syntax_context.functions.push(fn_item.clone());
                    }
                }
                FnType::ImplFn(impl_fn_item, impl_item) => {
                    let mut has_impl = false;
                    for has_impl_item in syntax_context.impls.iter_mut() {
                        if has_impl_item.get_item().eq(&impl_item.get_item()) {
                            has_impl_item.insert_function(&impl_fn_item);
                            has_impl = true;
                        }
                    }
                    if !has_impl {
                        let mut impl_item = impl_item.clone();
                        impl_item.insert_function(&impl_fn_item);
                        syntax_context.impls.push(impl_item);
                    }
                    let struct_item_string =
                        impl_item.get_struct_name().get_import_name().to_string();
                    let struct_item = structs.get(&struct_item_string);
                    if let Some(struct_item) = struct_item {
                        match &struct_item.struct_type {
                            StructType::Struct(struct_item) => {
                                if !syntax_context.structs.contains(&struct_item) {
                                    syntax_context.structs.push(struct_item.clone());
                                }
                            }
                            StructType::Enum(enum_item) => {
                                if !syntax_context.enums.contains(&enum_item) {
                                    syntax_context.enums.push(enum_item.clone());
                                }
                            }
                            StructType::Union(union_item) => {
                                if !syntax_context.unions.contains(&union_item) {
                                    syntax_context.unions.push(union_item.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                    // let trait_item_name = impl_item.get_trait_name();
                    // if let Some(trait_item_name) = Some(trait_item_name) {

                    // }
                }
                FnType::TraitFn(trait_fn_item, trait_item) => {
                    let mut has_trait = false;
                    for has_trait_item in syntax_context.traits.iter_mut() {
                        if has_trait_item.get_item().eq(&trait_item.get_item()) {
                            has_trait_item.insert_function(&trait_fn_item);
                            has_trait = true;
                        }
                    }
                    if !has_trait {
                        let mut trait_item = trait_item.clone();
                        trait_item.insert_function(&trait_fn_item);
                        syntax_context.traits.push(trait_item);
                    }
                    // let trait_item_string =
                    //     trait_item.get_trait_name().get_import_name().to_string();
                    // let trait_item = structs.get(&trait_item_string);
                    // if let Some(trait_item) = trait_item {
                    //     match &trait_item.struct_type {
                    //         // StructType::Struct(struct_item) => {
                    //         //     if !syntax_context.structs.contains(&struct_item) {
                    //         //         syntax_context.structs.push(struct_item.clone());
                    //         //     }
                    //         // }
                    //         // StructType::Enum(enum_item) => {
                    //         //     if !syntax_context.enums.contains(&enum_item) {
                    //         //         syntax_context.enums.push(enum_item.clone());
                    //         //     }
                    //         // }
                    //         // StructType::Union(union_item) => {
                    //         //     if !syntax_context.unions.contains(&union_item) {
                    //         //         syntax_context.unions.push(union_item.clone());
                    //         //     }
                    //         // }
                    //         // StructType::Trait(trait_item) => {
                    //         //     if !syntax_context.traits.contains(&trait_item) {
                    //         //         syntax_context.traits.push(trait_item);
                    //         //     }
                    //         // }
                    //     }
                    // }
                }
            }
        }
    }
    for a_type in data.types.iter() {
        let type_data = structs.get(a_type);
        if let Some(type_data) = type_data {
            match &type_data.struct_type {
                StructType::Struct(struct_item) => {
                    if !syntax_context.structs.contains(&struct_item) {
                        syntax_context.structs.push(struct_item.clone());
                    }
                }
                StructType::Enum(enum_item) => {
                    if !syntax_context.enums.contains(&enum_item) {
                        syntax_context.enums.push(enum_item.clone());
                    }
                }
                StructType::Union(union_item) => {
                    if !syntax_context.unions.contains(&union_item) {
                        syntax_context.unions.push(union_item.clone());
                    }
                }
                StructType::Trait(trait_item) => {
                    let mut has_trait = false;
                    for has_trait_item in syntax_context.traits.iter() {
                        if has_trait_item.get_item().eq(&trait_item.get_item()) {
                            has_trait = true;
                            break;
                        }
                    }
                    if !has_trait {
                        let mut trait_item = trait_item.clone();
                        syntax_context.traits.push(trait_item);
                    }
                }
            }
        }
    }
}

fn parse_callsandtypes(
    data: &mut CallsAndTypes,
    mod_trees: &Vec<String>,
    syntax: &mut SyntaxContext,
    fns: &HashMap<String, FnData>,
    structs: &HashMap<String, StructData>,
) {
    add_new_calls_and_types(data, mod_trees);
    get_syntax(data, syntax, fns, structs);
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

fn expand_use_tree(
    tree: &SynUseTree,
    visibility: &MyVisibility,
    expand_path: String,
    expanded_trees: &mut Vec<UseTree>,
) {
    match tree {
        SynUseTree::Path(use_path) => {
            let mut path_str = String::new();
            if expand_path != String::new() {
                path_str = expand_path + "::" + &use_path.ident.to_string();
            } else {
                path_str = use_path.ident.to_string();
            }
            expand_use_tree(&use_path.tree, visibility, path_str, expanded_trees);
        }
        SynUseTree::Name(use_name) => {
            let mut path_str = String::new();
            if expand_path != String::new() {
                path_str = expand_path + "::" + &use_name.ident.to_string();
            } else {
                path_str = use_name.ident.to_string();
            }
            let use_tree = UseTree::new(
                use_name.ident.to_string(),
                path_str,
                None,
                visibility.clone(),
            );
            expanded_trees.push(use_tree);
        }
        SynUseTree::Glob(_) => {
            let mut path_str = String::new();
            if expand_path != String::new() {
                path_str = expand_path + "::*";
            } else {
                path_str = "*".to_string();
            }
            let use_tree = UseTree::new("*".to_string(), path_str, None, visibility.clone());
            expanded_trees.push(use_tree);
        }
        SynUseTree::Rename(use_rename) => {
            let mut path_str = String::new();
            if expand_path != String::new() {
                path_str = expand_path + "::" + &use_rename.ident.to_string();
                let use_tree = UseTree::new(
                    use_rename.ident.to_string(),
                    path_str,
                    Some(use_rename.rename.to_string()),
                    visibility.clone(),
                );
                expanded_trees.push(use_tree);
            } else {
                path_str = use_rename.ident.to_string();
                let use_tree = UseTree::new(
                    use_rename.ident.to_string(),
                    path_str,
                    Some(use_rename.rename.to_string()),
                    visibility.clone(),
                );
                expanded_trees.push(use_tree);
            }
        }
        SynUseTree::Group(use_group) => {
            for item in use_group.items.iter() {
                expand_use_tree(item, visibility, expand_path.clone(), expanded_trees);
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
                    const_item.insert_visibility(parse_visibility(&item_const.vis));
                    syntax_context.consts.push(const_item);
                }
                Item::TraitAlias(item_trait_alias) => {
                    let mut trait_alias_item = TraitAliasItem::new();
                    let mut modified_item_trait_alias = item_trait_alias.clone();
                    modified_item_trait_alias.attrs =
                        delete_doc_attributes(&modified_item_trait_alias.attrs);
                    trait_alias_item.insert_item(&modified_item_trait_alias);
                    trait_alias_item.insert_visibility(parse_visibility(&item_trait_alias.vis));
                    syntax_context.trait_aliases.push(trait_alias_item);
                }
                Item::Use(item_use) => {
                    let mut use_item = UseItem::new();
                    let mut modified_item_use = item_use.clone();
                    modified_item_use.attrs = delete_doc_attributes(&modified_item_use.attrs);
                    use_item.insert_item(&modified_item_use);
                    let visibility = parse_visibility(&item_use.vis);
                    use_item.insert_visibility(visibility.clone());
                    syntax_context.uses.push(use_item);
                    let mut this_expanded_paths: Vec<UseTree> = Vec::new();
                    expand_use_tree(
                        &item_use.tree,
                        &visibility,
                        String::new(),
                        &mut this_expanded_paths,
                    );
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
                    mod_item.insert_visibility(parse_visibility(&item_mod.vis));
                    syntax_context.mods.push(mod_item);
                }
                Item::Static(item_static) => {
                    let mut static_item = StaticItem::new();
                    let mut modified_item_static = item_static.clone();
                    modified_item_static.attrs = delete_doc_attributes(&modified_item_static.attrs);
                    static_item.insert_item(&modified_item_static);
                    static_item.insert_visibility(parse_visibility(&item_static.vis));
                    syntax_context.statics.push(static_item);
                }
                Item::Type(item_type) => {
                    let mut type_item = TypeItem::new();
                    let mut modified_item_type = item_type.clone();
                    modified_item_type.attrs = delete_doc_attributes(&modified_item_type.attrs);
                    type_item.insert_item(&modified_item_type);
                    type_item.insert_visibility(parse_visibility(&item_type.vis));
                    syntax_context.types.push(type_item);
                }
                Item::Struct(item_struct) => {
                    let mut struct_item = StructItem::new();
                    struct_item.insert_struct_name(&item_struct.ident.to_string());
                    let mut modified_item_struct = item_struct.clone();
                    modified_item_struct.attrs = delete_doc_attributes(&modified_item_struct.attrs);
                    struct_item.insert_item(&modified_item_struct);
                    struct_item.insert_visibility(parse_visibility(&item_struct.vis));
                    let mut relative_types: Vec<String> = Vec::new();
                    visit_fields(&modified_item_struct.fields, &mut relative_types);
                    struct_item.insert_relative_types(relative_types);
                    syntax_context.structs.push(struct_item);
                }
                Item::Enum(item_enum) => {
                    let mut enum_item = EnumItem::new();
                    enum_item.insert_enum_name(&item_enum.ident.to_string());
                    let mut modified_item_enum = item_enum.clone();
                    modified_item_enum.attrs = delete_doc_attributes(&modified_item_enum.attrs);
                    enum_item.insert_item(&modified_item_enum);
                    enum_item.insert_visibility(parse_visibility(&item_enum.vis));
                    let mut relative_types: Vec<String> = Vec::new();
                    for variant in modified_item_enum.variants.iter() {
                        visit_fields(&variant.fields, &mut relative_types);
                    }
                    enum_item.insert_relative_types(relative_types);
                    syntax_context.enums.push(enum_item);
                }
                Item::Union(item_union) => {
                    let mut union_item = UnionItem::new();
                    union_item.insert_union_name(&item_union.ident.to_string());
                    let mut modified_item_union = item_union.clone();
                    modified_item_union.attrs = delete_doc_attributes(&modified_item_union.attrs);
                    union_item.insert_item(&modified_item_union);
                    union_item.insert_visibility(parse_visibility(&item_union.vis));
                    let mut relative_types: Vec<String> = Vec::new();
                    visit_fields_named(&modified_item_union.fields, &mut relative_types);
                    union_item.insert_relative_types(relative_types);
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
                    let mut import_names: Vec<String> = Vec::new();
                    let ty = *item_impl.self_ty.clone();
                    if let Type::Path(ty_path) = ty {
                        struct_name = ty_path.path.segments.last().unwrap().ident.to_string();
                        for segment in ty_path.path.segments.iter() {
                            import_names.push(segment.ident.to_string());
                        }
                    }
                    impl_item.insert_struct_name(&struct_name);
                    impl_item.insert_struct_import_name(&import_names.join("::"));
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
                        let mut import_names: Vec<String> = Vec::new();
                        for segment in item_impl.clone().trait_.unwrap().1.segments.iter() {
                            import_names.push(segment.ident.to_string());
                        }
                        impl_item.insert_trait_name(&trait_name);
                        impl_item.insert_trait_import_name(&import_names.join("::"));
                    }
                    for item in item_impl.items.iter() {
                        match item {
                            SynImplItem::Const(item_const) => {
                                let mut modified_item_const = item_const.clone();
                                modified_item_const.attrs =
                                    delete_doc_attributes(&modified_item_const.attrs);
                                let mut impl_const_item = ImplConstItem::new();
                                impl_const_item.insert_item(&modified_item_const);
                                impl_const_item
                                    .insert_visibility(parse_visibility(&item_const.vis));
                                impl_item.insert_const(&impl_const_item);
                            }
                            SynImplItem::Type(item_type) => {
                                let mut modified_item_type = item_type.clone();
                                modified_item_type.attrs =
                                    delete_doc_attributes(&modified_item_type.attrs);
                                let mut impl_type_item = ImplTypeItem::new();
                                impl_type_item.insert_item(&modified_item_type);
                                impl_type_item.insert_visibility(parse_visibility(&item_type.vis));
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
                                impl_fn_item.insert_visibility(parse_visibility(&item_fn.vis));
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
                    fn_item.insert_visibility(parse_visibility(&item_fn.vis));
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
                                let mut trait_fn_item = TraitFnItem::new();
                                trait_fn_item.insert_fn_name(&item_fn.sig.ident.to_string());
                                trait_fn_item
                                    .insert_complete_name_in_file(&trait_item.get_trait_name_str());
                                let mut modified_item_fn = item_fn.clone();
                                modified_item_fn.attrs =
                                    delete_doc_attributes(&modified_item_fn.attrs);
                                trait_fn_item.insert_item(&modified_item_fn);
                                let mut inside_items: Vec<Item> = Vec::new();
                                if let Some(block) = &item_fn.default {
                                    for stmt in block.stmts.iter() {
                                        if let Stmt::Item(stmt_item) = stmt {
                                            inside_items.push(stmt_item.clone());
                                        }
                                    }
                                }
                                trait_fn_item.insert_items(&inside_items);
                                trait_item.insert_function(&trait_fn_item);
                            }
                            _ => {}
                        }
                    }
                    trait_item.insert_visibility(parse_visibility(&item_trait.vis));
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

    pub fn change_fn_struct_enum_union_trait_name(&mut self, mod_tree: &String) {
        for fn_item in self.functions.iter_mut() {
            fn_item.insert_parent_mod_tree(mod_tree);
        }
        for struct_item in self.structs.iter_mut() {
            struct_item.insert_parent_mod_tree(mod_tree);
        }
        for enum_item in self.enums.iter_mut() {
            enum_item.insert_parent_mod_tree(mod_tree);
        }
        for union_item in self.unions.iter_mut() {
            union_item.insert_parent_mod_tree(mod_tree);
        }
        for trait_item in self.traits.iter_mut() {
            trait_item.insert_parent_mod_tree(mod_tree);
        }
    }

    pub fn change_use_trees(&mut self, mod_context: &Rc<RefCell<ModContext>>) {
        for use_tree in self.use_trees.iter_mut() {
            use_tree.change_use_tree(mod_context);
        }
    }

    pub fn get_pub_use(&self) -> Vec<UseTree> {
        let mut pub_uses: Vec<UseTree> = Vec::new();
        for use_tree in self.use_trees.iter() {
            match use_tree.get_visibility() {
                MyVisibility::Pri => {}
                _ => {
                    pub_uses.push(use_tree.clone());
                }
            }
        }
        pub_uses
    }

    pub fn has_fn_struct_enum_union_trait(&self, name: &String) -> bool {
        let mut r = false;
        for fn_item in self.functions.iter() {
            if fn_item.get_name().eq(name) {
                r = true;
                return r;
            }
        }
        for struct_item in self.structs.iter() {
            if struct_item.get_name().eq(name) {
                r = true;
                return r;
            }
        }
        for enum_item in self.enums.iter() {
            if enum_item.get_name().eq(name) {
                r = true;
                return r;
            }
        }
        for union_item in self.unions.iter() {
            if union_item.get_name().eq(name) {
                r = true;
                return r;
            }
        }
        for trait_item in self.traits.iter() {
            if trait_item.get_name().eq(name) {
                r = true;
                return r;
            }
        }
        return r;
    }

    pub fn change_impl_name(&mut self, mod_context: &Rc<RefCell<ModContext>>) {
        for impl_item in self.impls.iter_mut() {
            let mut name = impl_item.get_struct_name().clone();
            name.change_name_for_impl_struct_name(mod_context);
            impl_item.change_struct_name(&name);
            let mut name = impl_item.get_trait_name().clone();
            if let Some(_) = name {
                let mut name = name.as_mut().unwrap();
                name.change_name_for_impl_trait_name(mod_context);
                impl_item.change_trait_name(name);
            }
            impl_item.change_function_name(mod_context);
        }
        for trait_item in self.traits.iter_mut() {
            trait_item.change_function_name();
        }
    }

    pub fn get_struct_enum_union_name(&self, name: &String) -> Name {
        for struct_item in self.structs.iter() {
            if struct_item.get_name().eq(name) {
                return struct_item.get_struct_name().clone();
            }
        }
        for enum_item in self.enums.iter() {
            if enum_item.get_name().eq(name) {
                return enum_item.get_enum_name().clone();
            }
        }
        for union_item in self.unions.iter() {
            if union_item.get_name().eq(name) {
                return union_item.get_union_name().clone();
            }
        }
        for use_tree in self.use_trees.iter() {
            if use_tree.get_alias().is_some() {
                let alias_name = use_tree.get_alias().as_ref().unwrap();
                if name.eq(alias_name) {
                    let mut use_tree_name = Name::new(use_tree.get_name());
                    use_tree_name.insert_import_name(&use_tree.get_use_tree().to_string());
                    return use_tree_name;
                }
            }
        }
        for use_tree in self.use_trees.iter() {
            if name.eq(use_tree.get_name()) {
                let mut use_tree_name = Name::new(use_tree.get_name());
                use_tree_name.insert_import_name(&use_tree.get_use_tree().to_string());
                return use_tree_name;
            }
        }
        return Name::new(&"".to_string());
    }

    pub fn get_trait_name(&self, name: &String) -> Name {
        for trait_item in self.traits.iter() {
            if trait_item.get_name().eq(name) {
                return trait_item.get_trait_name().clone();
            }
        }
        for use_tree in self.use_trees.iter() {
            if use_tree.get_alias().is_some() {
                let alias_name = use_tree.get_alias().as_ref().unwrap();
                if name.eq(alias_name) {
                    let mut use_tree_name = Name::new(use_tree.get_name());
                    use_tree_name.insert_import_name(&use_tree.get_use_tree().to_string());
                    return use_tree_name;
                }
            }
        }
        for use_tree in self.use_trees.iter() {
            if name.eq(use_tree.get_name()) {
                let mut use_tree_name = Name::new(use_tree.get_name());
                use_tree_name.insert_import_name(&use_tree.get_use_tree().to_string());
                return use_tree_name;
            }
        }
        return Name::new(&"".to_string());
    }

    pub fn get_result(
        &self,
        fns: &mut HashMap<String, FnData>,
        structs: &mut HashMap<String, StructData>,
    ) {
        for function_item in self.functions.iter() {
            let fn_data = FnData {
                fn_name: function_item.get_name(),
                complete_fn_name: function_item.get_complete_name(),
                fn_type: FnType::Fn(function_item.clone()),
            };
            fns.insert(fn_data.complete_fn_name.clone(), fn_data);
        }
        for impl_item in self.impls.iter() {
            let mut empty_impl_item = impl_item.clone();
            empty_impl_item.clear();
            for function_item in impl_item.get_fns().iter() {
                let fn_data = FnData {
                    fn_name: function_item.get_name(),
                    complete_fn_name: function_item.get_complete_name(),
                    fn_type: FnType::ImplFn(function_item.clone(), empty_impl_item.clone()),
                };
                fns.insert(fn_data.complete_fn_name.clone(), fn_data);
            }
        }
        for trait_item in self.traits.iter() {
            let mut empty_trait_item = trait_item.clone();
            empty_trait_item.clear();
            for function_item in trait_item.get_fns().iter() {
                let fn_data = FnData {
                    fn_name: function_item.get_name(),
                    complete_fn_name: function_item.get_complete_name(),
                    fn_type: FnType::TraitFn(function_item.clone(), empty_trait_item.clone()),
                };
                fns.insert(fn_data.complete_fn_name.clone(), fn_data);
            }
            let struct_data = StructData {
                struct_name: trait_item.get_name(),
                complete_struct_name: trait_item.get_trait_name().get_import_name().to_string(),
                struct_type: StructType::Trait(empty_trait_item),
            };
            structs.insert(struct_data.complete_struct_name.clone(), struct_data);
        }
        for struct_item in self.structs.iter() {
            let struct_data = StructData {
                struct_name: struct_item.get_name(),
                complete_struct_name: struct_item.get_struct_name().get_import_name().to_string(),
                struct_type: StructType::Struct(struct_item.clone()),
            };
            structs.insert(struct_data.complete_struct_name.clone(), struct_data);
        }
        for enum_item in self.enums.iter() {
            let enum_data = StructData {
                struct_name: enum_item.get_name(),
                complete_struct_name: enum_item.get_enum_name().get_import_name().to_string(),
                struct_type: StructType::Enum(enum_item.clone()),
            };
            structs.insert(enum_data.complete_struct_name.clone(), enum_data);
        }
        for union_item in self.unions.iter() {
            let union_data = StructData {
                struct_name: union_item.get_name(),
                complete_struct_name: union_item.get_union_name().get_import_name().to_string(),
                struct_type: StructType::Union(union_item.clone()),
            };
            structs.insert(union_data.complete_struct_name.clone(), union_data);
        }
    }

    pub fn get_relative_types_for_struct(&self, name: &String, relative_types: &mut Vec<String>) {
        for struct_item in self.structs.iter() {
            if struct_item.get_name().eq(name) {
                *relative_types = struct_item.get_relative_types();
            }
        }
        for enum_item in self.enums.iter() {
            if enum_item.get_name().eq(name) {
                *relative_types = enum_item.get_relative_types();
            }
        }
        for union_item in self.unions.iter() {
            if union_item.get_name().eq(name) {
                *relative_types = union_item.get_relative_types();
            }
        }
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

    pub fn get_context(
        &self,
        output_path: &PathBuf,
        mod_tree: &String,
        mod_trees: &Vec<String>,
        fns: &HashMap<String, FnData>,
        structs: &HashMap<String, StructData>,
        crate_context: &CrateContext,
    ) {
        for function_item in self.functions.iter() {
            let complete_function_name =
                mod_tree.clone() + "::" + &function_item.get_complete_function_name_in_file();
            let call_file = output_path
                .join(String::from("callsandtypes/") + &complete_function_name + ".json");
            // println!("{}", call_file.to_string_lossy());
            let mut file = File::open(call_file);
            match file {
                Ok(mut file) => {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();
                    let mut data: CallsAndTypes = serde_json::from_str(&contents).unwrap();
                    let mut syntax_context = SyntaxContext::new();
                    // syntax_context.functions.push(function_item.clone());
                    data.calls.push(function_item.get_complete_name());
                    parse_callsandtypes(&mut data, mod_trees, &mut syntax_context, fns, structs);
                    let rs_file_name = complete_function_name.clone() + ".rs";
                    let output_file_path = output_path.join(rs_file_name);
                    let mut file = File::create(output_file_path).unwrap();
                    file.write_all(syntax_context.to_string().as_bytes())
                        .unwrap();

                    let directory_path = output_path.join("new_callsandtypes");
                    create_dir_all(&directory_path).unwrap();
                    let file_path = PathBuf::from(&directory_path)
                        .join(format!("{}.json", complete_function_name.clone()));
                    let mut file = File::create(&file_path).unwrap();
                    file.write_all(serde_json::to_string(&data).unwrap().as_bytes())
                        .unwrap();
                }
                Err(_) => {}
            }
        }
        for impl_item in self.impls.iter() {
            for function_item in impl_item.get_fns().iter() {
                let complete_function_name =
                    mod_tree.clone() + "::" + &function_item.get_complete_function_name_in_file();
                let call_file = output_path
                    .join(String::from("callsandtypes/") + &complete_function_name + ".json");
                let mut file = File::open(call_file);
                match file {
                    Ok(mut file) => {
                        let mut contents = String::new();
                        file.read_to_string(&mut contents).unwrap();
                        let mut data: CallsAndTypes = serde_json::from_str(&contents).unwrap();
                        let mut syntax_context = SyntaxContext::new();
                        data.calls.push(function_item.get_complete_name());
                        data.types
                            .push(impl_item.get_struct_name().get_import_name().to_string());
                        let mut relative_types: Vec<String> = Vec::new();
                        crate_context.get_relative_types_for_struct(
                            &impl_item.get_struct_name().get_import_name().to_string(),
                            &mut relative_types,
                        );
                        for relative_type in relative_types {
                            data.types.push(relative_type);
                        }
                        if let Some(trait_name) = impl_item.get_trait_name() {
                            data.types.push(trait_name.get_import_name().to_string());
                        }
                        parse_callsandtypes(
                            &mut data,
                            mod_trees,
                            &mut syntax_context,
                            fns,
                            structs,
                        );
                        let rs_file_name = complete_function_name.clone() + ".rs";
                        let output_file_path = output_path.join(rs_file_name);
                        let mut file = File::create(output_file_path).unwrap();
                        file.write_all(syntax_context.to_string().as_bytes())
                            .unwrap();

                        let directory_path = output_path.join("new_callsandtypes");
                        create_dir_all(&directory_path).unwrap();
                        let file_path = PathBuf::from(&directory_path)
                            .join(format!("{}.json", complete_function_name.clone()));
                        let mut file = File::create(&file_path).unwrap();
                        file.write_all(serde_json::to_string(&data).unwrap().as_bytes())
                            .unwrap();
                    }
                    Err(_) => {}
                }
            }
        }
        for trait_item in self.traits.iter() {
            for function_item in trait_item.get_fns().iter() {
                let complete_function_name =
                    mod_tree.clone() + "::" + &function_item.get_complete_function_name_in_file();
                let call_file = output_path
                    .join(String::from("callsandtypes/") + &complete_function_name + ".json");
                let mut file = File::open(call_file);
                match file {
                    Ok(mut file) => {
                        let mut contents = String::new();
                        file.read_to_string(&mut contents).unwrap();
                        let mut data: CallsAndTypes = serde_json::from_str(&contents).unwrap();
                        let mut syntax_context = SyntaxContext::new();
                        data.calls.push(function_item.get_complete_name());
                        data.types.push(trait_item.get_name());
                        parse_callsandtypes(
                            &mut data,
                            mod_trees,
                            &mut syntax_context,
                            fns,
                            structs,
                        );
                        let rs_file_name = complete_function_name.clone() + ".rs";
                        let output_file_path = output_path.join(rs_file_name);
                        let mut file = File::create(output_file_path).unwrap();
                        file.write_all(syntax_context.to_string().as_bytes())
                            .unwrap();

                        let directory_path = output_path.join("new_callsandtypes");
                        create_dir_all(&directory_path).unwrap();
                        let file_path = PathBuf::from(&directory_path)
                            .join(format!("{}.json", complete_function_name.clone()));
                        let mut file = File::create(&file_path).unwrap();
                        file.write_all(serde_json::to_string(&data).unwrap().as_bytes())
                            .unwrap();
                    }
                    Err(_) => {}
                }
            }
        }
    }

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
