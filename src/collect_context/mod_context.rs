use std::{
    cell::RefCell,
    fmt,
    fs::{self, read_to_string},
    path::PathBuf,
    process,
    rc::Rc,
};

use syn::{parse_file, Item};

use super::syntax_context::SyntaxContext;

#[derive(Debug, Clone)]
pub struct ModModInfo {
    mod_name: String,
    mod_tree: String,
    file_path: PathBuf,
    parent_directory_path: PathBuf,
    mod_file_directory_path: Option<PathBuf>,
}

impl ModModInfo {
    pub fn new() -> Self {
        ModModInfo {
            mod_name: String::new(),
            mod_tree: String::new(),
            file_path: PathBuf::new(),
            parent_directory_path: PathBuf::new(),
            mod_file_directory_path: None,
        }
    }

    pub fn insert_mod_name(&mut self, mod_name: &String) {
        self.mod_name = mod_name.clone();
    }

    pub fn insert_parent_mod_tree(&mut self, parent_mod_tree: &String) {
        if self.mod_name.eq("") {
            eprintln!("Mod name is empty!");
            process::exit(6);
        }
        if parent_mod_tree.eq("") {
            self.mod_tree = self.mod_name.clone();
        } else {
            self.mod_tree = parent_mod_tree.clone() + "::" + &self.mod_name;
        }
    }

    pub fn insert_file_path(&mut self, file_path: &PathBuf) {
        self.file_path = file_path.clone();
    }

    pub fn insert_parent_directory_path(&mut self, parent_directory_path: &PathBuf) {
        self.parent_directory_path = parent_directory_path.clone();
    }

    pub fn insert_mod_file_directory_path(&mut self, mod_file_directory_path: &PathBuf) {
        self.mod_file_directory_path = Some(mod_file_directory_path.clone());
    }

    fn get_mod_name(&self) -> String {
        return self.mod_name.clone();
    }

    fn get_parent_directory_path(&self) -> PathBuf {
        return self.parent_directory_path.clone();
    }

    fn get_mod_tree(&self) -> String {
        return self.mod_tree.clone();
    }
}

#[derive(Debug, Clone)]
pub struct FunctionModInfo {
    function_name: String,
    mod_tree: String,
}

impl FunctionModInfo {
    pub fn new() -> Self {
        FunctionModInfo {
            function_name: String::new(),
            mod_tree: String::new(),
        }
    }

    fn insert_function_name(&mut self, function_name: &String) {
        self.function_name = function_name.clone();
    }

    fn insert_parent_mod_tree(&mut self, parent_mod_tree: &String) {
        if self.function_name.eq("") {
            eprintln!("Mod name is empty!");
            process::exit(6);
        }
        if parent_mod_tree.eq("") {
            self.mod_tree = self.function_name.clone();
        } else {
            self.mod_tree = parent_mod_tree.clone() + "::" + &self.function_name;
        }
    }

    fn get_function_name(&self) -> String {
        return self.function_name.clone();
    }

    fn get_mod_tree(&self) -> String {
        return self.mod_tree.clone();
    }
}

#[derive(Debug, Clone)]
pub enum ModInfo {
    Mod(ModModInfo),
    Fn(FunctionModInfo),
}

impl ModInfo {
    pub fn new() -> Self {
        ModInfo::Mod(ModModInfo::new())
    }

    fn get_parent_directory_path(&self) -> PathBuf {
        if let ModInfo::Mod(mod_mod_info) = self {
            return mod_mod_info.get_parent_directory_path();
        } else {
            eprintln!("Can not get parent directory path!");
            process::exit(8);
        }
    }

    fn get_mod_name(&self) -> String {
        match self {
            ModInfo::Mod(mod_mod_info) => {
                return mod_mod_info.get_mod_name();
            }
            ModInfo::Fn(function_mod_info) => {
                return function_mod_info.get_function_name();
            }
        }
    }

    fn get_mod_tree(&self) -> String {
        match self {
            ModInfo::Mod(mod_mod_info) => {
                return mod_mod_info.get_mod_tree();
            }
            ModInfo::Fn(function_mod_info) => {
                return function_mod_info.get_mod_tree();
            }
        }
    }
}

#[derive(Clone)]
pub struct ModContext {
    mod_info: ModInfo,
    syntax_context: SyntaxContext,
    sub_mods: Vec<Rc<RefCell<ModContext>>>,
    parent_mod: Option<Rc<RefCell<ModContext>>>,
    crate_mod: Option<Rc<RefCell<ModContext>>>,
}

impl fmt::Debug for ModContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ModContext {{\n mod_info: {:#?},\n syntax_context: {:#?},\n sub_mods: {:#?}\n }}\n",
            self.mod_info, self.syntax_context, self.sub_mods
        )
    }
}

impl ModContext {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(ModContext {
            mod_info: ModInfo::new(),
            syntax_context: SyntaxContext::new(),
            sub_mods: Vec::new(),
            parent_mod: None,
            crate_mod: None,
        }))
    }

    pub fn parse_from_items(
        parent: &Rc<RefCell<ModContext>>,
        items: &Vec<Item>,
        crate_mod: &Option<Rc<RefCell<ModContext>>>,
    ) {
        parent.borrow_mut().syntax_context = SyntaxContext::from_items(items);
        let inline_mods = parent.borrow().syntax_context.get_inline_mods();
        let no_inline_mods = parent.borrow().syntax_context.get_no_inline_mods();
        let functions_with_items = parent.borrow().syntax_context.get_functions_with_items();
        for inline_mod in inline_mods.iter() {
            let mut mod_mod_info = ModModInfo::new();
            mod_mod_info.insert_mod_name(&inline_mod.get_mod_name());
            mod_mod_info.insert_parent_mod_tree(&parent.borrow().mod_info.get_mod_tree());
            let mod_info = ModInfo::Mod(mod_mod_info);
            let sub_mod = ModContext::new();
            sub_mod.borrow_mut().insert_mod_info(&mod_info);
            ModContext::parse_from_items(&sub_mod, &inline_mod.get_items(), crate_mod);
            sub_mod.borrow_mut().parent_mod = Some(Rc::clone(parent));
            sub_mod.borrow_mut().crate_mod = Some(Rc::clone(crate_mod.as_ref().unwrap()));
            parent.borrow_mut().sub_mods.push(sub_mod);
        }
        for function_with_item in functions_with_items.iter() {
            let mut function_mod_info = FunctionModInfo::new();
            function_mod_info
                .insert_function_name(&&function_with_item.get_complete_function_name_in_file());
            function_mod_info.insert_parent_mod_tree(&parent.borrow().mod_info.get_mod_tree());
            let mod_info = ModInfo::Fn(function_mod_info);
            let sub_mod = ModContext::new();
            sub_mod.borrow_mut().insert_mod_info(&mod_info);
            ModContext::parse_from_items(&sub_mod, &function_with_item.get_items(), crate_mod);
            sub_mod.borrow_mut().crate_mod = Some(Rc::clone(crate_mod.as_ref().unwrap()));
            parent.borrow_mut().sub_mods.push(sub_mod);
        }
        for no_inline_mod in no_inline_mods.iter() {
            let mut mod_mod_info = ModModInfo::new();
            mod_mod_info.insert_mod_name(&no_inline_mod.get_mod_name());
            mod_mod_info.insert_parent_mod_tree(&parent.borrow().mod_info.get_mod_tree());
            mod_mod_info.insert_parent_directory_path(
                &parent.borrow().mod_info.get_parent_directory_path(),
            );
            let mut rs_file_name = String::new();
            let mut single_file_path = PathBuf::new();
            let mut mod_directory_path = PathBuf::new();
            let mut mod_file_path = PathBuf::new();
            let file_name = no_inline_mod.get_file_name();
            if let None = file_name {
                rs_file_name = mod_mod_info.get_mod_name() + ".rs";
                single_file_path = mod_mod_info.get_parent_directory_path().join(rs_file_name);
                mod_directory_path = mod_mod_info
                    .get_parent_directory_path()
                    .join(mod_mod_info.get_mod_name());
                mod_file_path = mod_directory_path.join("mod.rs");
                if fs::exists(&mod_file_path).unwrap() {
                    mod_mod_info.insert_file_path(&mod_file_path);
                    mod_mod_info.insert_parent_directory_path(&mod_directory_path);
                    mod_mod_info.insert_mod_file_directory_path(&mod_directory_path);
                    let code = read_to_string(&mod_file_path).unwrap();
                    let syntax = parse_file(&code).unwrap();
                    let mod_info = ModInfo::Mod(mod_mod_info);
                    let sub_mod = ModContext::new();
                    sub_mod.borrow_mut().insert_mod_info(&mod_info);
                    ModContext::parse_from_items(&sub_mod, &syntax.items, crate_mod);
                    sub_mod.borrow_mut().crate_mod = Some(Rc::clone(crate_mod.as_ref().unwrap()));
                    parent.borrow_mut().sub_mods.push(sub_mod);
                } else if fs::exists(&single_file_path).unwrap() {
                    if fs::exists(&mod_directory_path).unwrap() {
                        mod_mod_info.insert_file_path(&single_file_path);
                        mod_mod_info.insert_parent_directory_path(&mod_directory_path);
                        mod_mod_info.insert_mod_file_directory_path(&mod_directory_path);
                        let code = read_to_string(&single_file_path).unwrap();
                        let syntax = parse_file(&code).unwrap();
                        let mod_info = ModInfo::Mod(mod_mod_info);
                        let sub_mod = ModContext::new();
                        sub_mod.borrow_mut().insert_mod_info(&mod_info);
                        ModContext::parse_from_items(&sub_mod, &syntax.items, crate_mod);
                        sub_mod.borrow_mut().crate_mod =
                            Some(Rc::clone(crate_mod.as_ref().unwrap()));
                        parent.borrow_mut().sub_mods.push(sub_mod);
                    } else {
                        mod_mod_info.insert_file_path(&single_file_path);
                        let code = read_to_string(&single_file_path).unwrap();
                        let syntax = parse_file(&code).unwrap();
                        let mod_info = ModInfo::Mod(mod_mod_info);
                        let sub_mod = ModContext::new();
                        sub_mod.borrow_mut().insert_mod_info(&mod_info);
                        ModContext::parse_from_items(&sub_mod, &syntax.items, crate_mod);
                        sub_mod.borrow_mut().crate_mod =
                            Some(Rc::clone(crate_mod.as_ref().unwrap()));
                        parent.borrow_mut().sub_mods.push(sub_mod);
                    }
                } else {
                    eprintln!("Wrong when parse mod path in the crate!");
                    process::exit(9);
                }
            } else {
                let file_name_path = PathBuf::from(file_name.unwrap());
                let real_file_name = file_name_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                if real_file_name.eq("mod.rs") {
                    let mod_path_name = file_name_path
                        .parent()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    mod_directory_path = parent
                        .borrow()
                        .mod_info
                        .get_parent_directory_path()
                        .join(mod_path_name);
                    mod_file_path = mod_directory_path.join("mod.rs");
                    if fs::exists(&mod_file_path).unwrap() {
                        mod_mod_info.insert_file_path(&mod_file_path);
                        mod_mod_info.insert_parent_directory_path(&mod_directory_path);
                        mod_mod_info.insert_mod_file_directory_path(&mod_directory_path);
                        let code = read_to_string(&mod_file_path).unwrap();
                        let syntax = parse_file(&code).unwrap();
                        let mod_info = ModInfo::Mod(mod_mod_info);
                        let sub_mod = ModContext::new();
                        sub_mod.borrow_mut().insert_mod_info(&mod_info);
                        ModContext::parse_from_items(&sub_mod, &syntax.items, crate_mod);
                        sub_mod.borrow_mut().crate_mod =
                            Some(Rc::clone(crate_mod.as_ref().unwrap()));
                        parent.borrow_mut().sub_mods.push(sub_mod);
                    } else {
                        eprintln!("Wrong when parse mod path in the crate!");
                        process::exit(10);
                    }
                } else {
                    let mod_path_name = file_name_path
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    mod_directory_path = parent
                        .borrow()
                        .mod_info
                        .get_parent_directory_path()
                        .join(mod_path_name);
                    mod_file_path = parent
                        .borrow()
                        .mod_info
                        .get_parent_directory_path()
                        .join(file_name_path);
                    if fs::exists(&mod_file_path).unwrap() {
                        if fs::exists(&mod_directory_path).unwrap() {
                            mod_mod_info.insert_file_path(&mod_file_path);
                            mod_mod_info.insert_parent_directory_path(&mod_directory_path);
                            mod_mod_info.insert_mod_file_directory_path(&mod_directory_path);
                            let code = read_to_string(&mod_file_path).unwrap();
                            let syntax = parse_file(&code).unwrap();
                            let mod_info = ModInfo::new();
                            let sub_mod = ModContext::new();
                            sub_mod.borrow_mut().insert_mod_info(&mod_info);
                            ModContext::parse_from_items(&sub_mod, &syntax.items, crate_mod);
                            sub_mod.borrow_mut().crate_mod =
                                Some(Rc::clone(crate_mod.as_ref().unwrap()));
                            parent.borrow_mut().sub_mods.push(sub_mod);
                        } else {
                            mod_mod_info.insert_file_path(&mod_file_path);
                            let code = read_to_string(&mod_file_path).unwrap();
                            let syntax = parse_file(&code).unwrap();
                            let mod_info = ModInfo::new();
                            let sub_mod = ModContext::new();
                            sub_mod.borrow_mut().insert_mod_info(&mod_info);
                            ModContext::parse_from_items(&sub_mod, &syntax.items, crate_mod);
                            sub_mod.borrow_mut().crate_mod =
                                Some(Rc::clone(crate_mod.as_ref().unwrap()));
                            parent.borrow_mut().sub_mods.push(sub_mod);
                        }
                    } else {
                        eprintln!("Wrong when parse mod path in the crate!");
                        process::exit(11);
                    }
                }
            }
        }
    }

    pub fn insert_mod_info(&mut self, mod_info: &ModInfo) {
        self.mod_info = mod_info.clone();
    }

    pub fn get_all_mod_trees(&self, mod_trees: &mut Vec<String>) {
        mod_trees.push(self.mod_info.get_mod_tree());
        for sub_mod in self.sub_mods.iter() {
            sub_mod.borrow().get_all_mod_trees(mod_trees);
        }
    }

    pub fn get_complete_function_names(&self, function_names: &mut Vec<String>) {
        let single_function_names = self.syntax_context.get_all_in_file_function_names();
        let mod_tree = self.mod_info.get_mod_tree();
        for single_function_name in single_function_names.iter() {
            function_names.push(mod_tree.clone() + "::" + single_function_name);
        }
        for sub_mod in self.sub_mods.iter() {
            sub_mod.borrow().get_complete_function_names(function_names);
        }
    }

    // pub fn get_all_item(&self, item_name: &String, syntax_context: &mut SyntaxContext) {
    //     let one_syntax_context = self.syntax_context.get_item(item_name);
    //     syntax_context.extend_with_other(&one_syntax_context);
    //     for sub_mod in self.sub_mods.iter() {
    //         sub_mod.borrow().get_all_item(item_name, syntax_context);
    //     }
    // }

    // pub fn get_all_simplified_item(&self, item_name: &String, syntax_context: &mut SyntaxContext) {
    //     let one_syntax_context = self.syntax_context.get_simplified_item(item_name);
    //     syntax_context.extend_with_other(&one_syntax_context);
    //     for sub_mod in self.sub_mods.iter() {
    //         sub_mod
    //             .borrow()
    //             .get_all_simplified_item(item_name, syntax_context);
    //     }
    // }

    // pub fn get_all_context(&self, output_path: &PathBuf, main_mod_contexts: &Vec<ModContext>) {
    //     self.syntax_context.get_context(
    //         output_path,
    //         &self.mod_info.get_mod_tree(),
    //         main_mod_contexts,
    //     );
    //     for sub_mod in self.sub_mods.iter() {
    //         sub_mod
    //             .borrow()
    //             .get_all_context(output_path, main_mod_contexts);
    //     }
    // }
}
