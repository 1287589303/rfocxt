trait SEUGetApplications {
    fn get_impls(&self) -> Vec<ImplItem>;
    fn get_traits_impls(&self) -> Vec<ImplItem>;
    fn get_applications(&self) -> Vec<String> {
        let mut applications: Vec<String> = Vec::new();
        for impl_item in self.get_impls().iter() {
            for function_item in impl_item.functions.iter() {
                applications.extend(function_item.applications.clone());
            }
        }
        for trait_impl in self.get_traits_impls().iter() {
            for function_item in trait_impl.functions.iter() {
                applications.extend(function_item.applications.clone());
            }
        }
        applications
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
    pub fn to_string(&self) -> String {
        let mut re = String::new();
        for syn_file in self.syn_files.iter() {
            re = re
                + "//"
                + syn_file.file_name.as_str()
                + "\n"
                + syn_file.to_string().as_str()
                + "\n";
        }
        re
    }
    pub fn get_all_names(&mut self) {
        for syn_file in self.syn_files.iter() {
            self.all_names.extend(
                syn_file
                    .structs
                    .iter()
                    .map(|struct_item| struct_item.clone().struct_name),
            );
            self.all_names.extend(
                syn_file
                    .enums
                    .iter()
                    .map(|enum_item| enum_item.clone().enum_name),
            );
            self.all_names.extend(
                syn_file
                    .unions
                    .iter()
                    .map(|union_item| union_item.clone().union_name),
            );
            self.all_names.extend(
                syn_file
                    .functions
                    .iter()
                    .map(|function_item| function_item.clone().function_name),
            );
        }
    }
    pub fn change_applications(&mut self) {
        for syn_file in self.syn_files.iter_mut() {
            syn_file.change_applications(&self.all_names);
        }
    }
    fn get_context_item(&self, application: String) -> Option<ContextItem> {
        for syn_file in self.syn_files.iter() {
            for struct_item in syn_file.structs.iter() {
                if struct_item.struct_name == application {
                    return Some(ContextItem::Struct(struct_item.clone()));
                }
            }
            for enum_item in syn_file.enums.iter() {
                if enum_item.enum_name == application {
                    return Some(ContextItem::Enum(enum_item.clone()));
                }
            }
            for union_item in syn_file.unions.iter() {
                if union_item.union_name == application {
                    return Some(ContextItem::Union(union_item.clone()));
                }
            }
            for trait_item in syn_file.traits.iter() {
                if trait_item.trait_name == application {
                    return Some(ContextItem::Trait(trait_item.clone()));
                }
            }
        }
        None
    }
    pub fn get_all_context(&self, project_path: PathBuf) {
        let output_path = project_path.join("context");
        for syn_file in self.syn_files.iter() {
            let syn_file_path = output_path.clone().join(syn_file.clone().file_name);
            for struct_item in syn_file.structs.iter() {
                let struct_path = syn_file_path.clone().join(struct_item.clone().struct_name);
                for impl_item in struct_item.impls.iter() {
                    for function_item in impl_item.functions.iter() {
                        let function_path = struct_path
                            .clone()
                            .join(function_item.clone().function_name);
                        fs::create_dir_all(function_path.clone()).unwrap();
                        let mut out_syn_file = SynFile::new();
                        let mut remain_applications: Vec<String> = Vec::new();
                        remain_applications = function_item.applications.clone();
                        let mut already_applications: Vec<String> = Vec::new();
                        already_applications.push(struct_item.struct_name.clone());
                        out_syn_file.structs.push(struct_item.clone());
                        while !remain_applications.is_empty() {
                            let application = remain_applications.pop().unwrap();
                            if !already_applications.contains(&application) {
                                already_applications.push(application.clone());
                                let context_item = self.get_context_item(application);
                                if let Some(context_item) = context_item {
                                    match context_item {
                                        ContextItem::Enum(enum_item) => {
                                            out_syn_file.enums.push(enum_item.clone());
                                            remain_applications
                                                .extend(enum_item.get_applications());
                                        }
                                        ContextItem::Union(union_item) => {
                                            out_syn_file.unions.push(union_item.clone());
                                            remain_applications
                                                .extend(union_item.get_applications());
                                        }
                                        ContextItem::Struct(struct_item) => {
                                            out_syn_file.structs.push(struct_item.clone());
                                            remain_applications
                                                .extend(struct_item.get_applications());
                                        }
                                        ContextItem::Trait(trait_item) => {
                                            out_syn_file.traits.push(trait_item.clone());
                                            remain_applications
                                                .extend(trait_item.get_applications());
                                        }
                                    }
                                }
                            }
                        }
                        let file_path = function_path.clone().join("context.rs");
                        let mut file = fs::File::create(file_path).unwrap();
                        file.write_all(out_syn_file.to_string().as_bytes()).unwrap();
                    }
                }
                for i in 0..struct_item.traits.len() {
                    let trait_path = struct_path.join(struct_item.traits[i].clone());
                    for function_item in struct_item.traits_impls[i].functions.iter() {
                        let function_path =
                            trait_path.clone().join(function_item.clone().function_name);
                        fs::create_dir_all(function_path.clone()).unwrap();
                        let mut out_syn_file = SynFile::new();
                        let mut remain_applications: Vec<String> = Vec::new();
                        remain_applications = function_item.applications.clone();
                        let mut already_applications: Vec<String> = Vec::new();
                        already_applications.push(struct_item.struct_name.clone());
                        out_syn_file.structs.push(struct_item.clone());
                        while !remain_applications.is_empty() {
                            let application = remain_applications.pop().unwrap();
                            if !already_applications.contains(&application) {
                                already_applications.push(application.clone());
                                let context_item = self.get_context_item(application);
                                if let Some(context_item) = context_item {
                                    match context_item {
                                        ContextItem::Enum(enum_item) => {
                                            out_syn_file.enums.push(enum_item.clone());
                                            remain_applications
                                                .extend(enum_item.get_applications());
                                        }
                                        ContextItem::Union(union_item) => {
                                            out_syn_file.unions.push(union_item.clone());
                                            remain_applications
                                                .extend(union_item.get_applications());
                                        }
                                        ContextItem::Struct(struct_item) => {
                                            out_syn_file.structs.push(struct_item.clone());
                                            remain_applications
                                                .extend(struct_item.get_applications());
                                        }
                                        ContextItem::Trait(trait_item) => {
                                            out_syn_file.traits.push(trait_item.clone());
                                            remain_applications
                                                .extend(trait_item.get_applications());
                                        }
                                    }
                                }
                            }
                        }
                        let file_path = function_path.clone().join("context.rs");
                        let mut file = fs::File::create(file_path).unwrap();
                        file.write_all(out_syn_file.to_string().as_bytes()).unwrap();
                    }
                }
            }
            for enum_item in syn_file.enums.iter() {
                let enum_path = syn_file_path.clone().join(enum_item.clone().enum_name);
                for impl_item in enum_item.impls.iter() {
                    for function_item in impl_item.functions.iter() {
                        let function_path =
                            enum_path.clone().join(function_item.clone().function_name);
                        fs::create_dir_all(function_path.clone()).unwrap();
                        let mut out_syn_file = SynFile::new();
                        let mut remain_applications: Vec<String> = Vec::new();
                        remain_applications = function_item.applications.clone();
                        let mut already_applications: Vec<String> = Vec::new();
                        already_applications.push(enum_item.enum_name.clone());
                        out_syn_file.enums.push(enum_item.clone());
                        while !remain_applications.is_empty() {
                            let application = remain_applications.pop().unwrap();
                            if !already_applications.contains(&application) {
                                already_applications.push(application.clone());
                                let context_item = self.get_context_item(application);
                                if let Some(context_item) = context_item {
                                    match context_item {
                                        ContextItem::Enum(enum_item) => {
                                            out_syn_file.enums.push(enum_item.clone());
                                            remain_applications
                                                .extend(enum_item.get_applications());
                                        }
                                        ContextItem::Union(union_item) => {
                                            out_syn_file.unions.push(union_item.clone());
                                            remain_applications
                                                .extend(union_item.get_applications());
                                        }
                                        ContextItem::Struct(struct_item) => {
                                            out_syn_file.structs.push(struct_item.clone());
                                            remain_applications
                                                .extend(struct_item.get_applications());
                                        }
                                        ContextItem::Trait(trait_item) => {
                                            out_syn_file.traits.push(trait_item.clone());
                                            remain_applications
                                                .extend(trait_item.get_applications());
                                        }
                                    }
                                }
                            }
                        }
                        let file_path = function_path.clone().join("context.rs");
                        let mut file = fs::File::create(file_path).unwrap();
                        file.write_all(out_syn_file.to_string().as_bytes()).unwrap();
                    }
                }
                for i in 0..enum_item.traits.len() {
                    let trait_path = enum_path.join(enum_item.traits[i].clone());
                    for function_item in enum_item.traits_impls[i].functions.iter() {
                        let function_path =
                            trait_path.clone().join(function_item.clone().function_name);
                        fs::create_dir_all(function_path.clone()).unwrap();
                        let mut out_syn_file = SynFile::new();
                        let mut remain_applications: Vec<String> = Vec::new();
                        remain_applications = function_item.applications.clone();
                        let mut already_applications: Vec<String> = Vec::new();
                        already_applications.push(enum_item.enum_name.clone());
                        out_syn_file.enums.push(enum_item.clone());
                        while !remain_applications.is_empty() {
                            let application = remain_applications.pop().unwrap();
                            if !already_applications.contains(&application) {
                                already_applications.push(application.clone());
                                let context_item = self.get_context_item(application);
                                if let Some(context_item) = context_item {
                                    match context_item {
                                        ContextItem::Enum(enum_item) => {
                                            out_syn_file.enums.push(enum_item.clone());
                                            remain_applications
                                                .extend(enum_item.get_applications());
                                        }
                                        ContextItem::Union(union_item) => {
                                            out_syn_file.unions.push(union_item.clone());
                                            remain_applications
                                                .extend(union_item.get_applications());
                                        }
                                        ContextItem::Struct(struct_item) => {
                                            out_syn_file.structs.push(struct_item.clone());
                                            remain_applications
                                                .extend(struct_item.get_applications());
                                        }
                                        ContextItem::Trait(trait_item) => {
                                            out_syn_file.traits.push(trait_item.clone());
                                            remain_applications
                                                .extend(trait_item.get_applications());
                                        }
                                    }
                                }
                            }
                        }
                        let file_path = function_path.clone().join("context.rs");
                        let mut file = fs::File::create(file_path).unwrap();
                        file.write_all(out_syn_file.to_string().as_bytes()).unwrap();
                    }
                }
            }
            for union_item in syn_file.unions.iter() {
                let union_path = syn_file_path.clone().join(union_item.clone().union_name);
                for impl_item in union_item.impls.iter() {
                    for function_item in impl_item.functions.iter() {
                        let function_path =
                            union_path.clone().join(function_item.clone().function_name);
                        fs::create_dir_all(function_path.clone()).unwrap();
                        let mut out_syn_file = SynFile::new();
                        let mut remain_applications: Vec<String> = Vec::new();
                        remain_applications = function_item.applications.clone();
                        let mut already_applications: Vec<String> = Vec::new();
                        already_applications.push(union_item.union_name.clone());
                        out_syn_file.unions.push(union_item.clone());
                        while !remain_applications.is_empty() {
                            let application = remain_applications.pop().unwrap();
                            if !already_applications.contains(&application) {
                                already_applications.push(application.clone());
                                let context_item = self.get_context_item(application);
                                if let Some(context_item) = context_item {
                                    match context_item {
                                        ContextItem::Enum(enum_item) => {
                                            out_syn_file.enums.push(enum_item.clone());
                                            remain_applications
                                                .extend(enum_item.get_applications());
                                        }
                                        ContextItem::Union(union_item) => {
                                            out_syn_file.unions.push(union_item.clone());
                                            remain_applications
                                                .extend(union_item.get_applications());
                                        }
                                        ContextItem::Struct(struct_item) => {
                                            out_syn_file.structs.push(struct_item.clone());
                                            remain_applications
                                                .extend(struct_item.get_applications());
                                        }
                                        ContextItem::Trait(trait_item) => {
                                            out_syn_file.traits.push(trait_item.clone());
                                            remain_applications
                                                .extend(trait_item.get_applications());
                                        }
                                    }
                                }
                            }
                        }
                        let file_path = function_path.clone().join("context.rs");
                        let mut file = fs::File::create(file_path).unwrap();
                        file.write_all(out_syn_file.to_string().as_bytes()).unwrap();
                    }
                }
                for i in 0..union_item.traits.len() {
                    let trait_path = union_path.join(union_item.traits[i].clone());
                    for function_item in union_item.traits_impls[i].functions.iter() {
                        let function_path =
                            trait_path.clone().join(function_item.clone().function_name);
                        fs::create_dir_all(function_path.clone()).unwrap();
                        let mut out_syn_file = SynFile::new();
                        let mut remain_applications: Vec<String> = Vec::new();
                        remain_applications = function_item.applications.clone();
                        let mut already_applications: Vec<String> = Vec::new();
                        already_applications.push(union_item.union_name.clone());
                        out_syn_file.unions.push(union_item.clone());
                        while !remain_applications.is_empty() {
                            let application = remain_applications.pop().unwrap();
                            if !already_applications.contains(&application) {
                                already_applications.push(application.clone());
                                let context_item = self.get_context_item(application);
                                if let Some(context_item) = context_item {
                                    match context_item {
                                        ContextItem::Enum(enum_item) => {
                                            out_syn_file.enums.push(enum_item.clone());
                                            remain_applications
                                                .extend(enum_item.get_applications());
                                        }
                                        ContextItem::Union(union_item) => {
                                            out_syn_file.unions.push(union_item.clone());
                                            remain_applications
                                                .extend(union_item.get_applications());
                                        }
                                        ContextItem::Struct(struct_item) => {
                                            out_syn_file.structs.push(struct_item.clone());
                                            remain_applications
                                                .extend(struct_item.get_applications());
                                        }
                                        ContextItem::Trait(trait_item) => {
                                            out_syn_file.traits.push(trait_item.clone());
                                            remain_applications
                                                .extend(trait_item.get_applications());
                                        }
                                    }
                                }
                            }
                        }
                        let file_path = function_path.clone().join("context.rs");
                        let mut file = fs::File::create(file_path).unwrap();
                        file.write_all(out_syn_file.to_string().as_bytes()).unwrap();
                    }
                }
            }
            for function_item in syn_file.functions.iter() {
                let function_path = syn_file_path
                    .clone()
                    .join(function_item.clone().function_name);
                fs::create_dir_all(function_path.clone()).unwrap();
                let mut out_syn_file = SynFile::new();
                let mut remain_applications: Vec<String> = Vec::new();
                remain_applications = function_item.applications.clone();
                let mut already_applications: Vec<String> = Vec::new();
                already_applications.push(function_item.function_name.clone());
                out_syn_file.functions.push(function_item.clone());
                while !remain_applications.is_empty() {
                    let application = remain_applications.pop().unwrap();
                    if !already_applications.contains(&application) {
                        already_applications.push(application.clone());
                        let context_item = self.get_context_item(application);
                        if let Some(context_item) = context_item {
                            match context_item {
                                ContextItem::Enum(enum_item) => {
                                    out_syn_file.enums.push(enum_item.clone());
                                    remain_applications.extend(enum_item.get_applications());
                                }
                                ContextItem::Union(union_item) => {
                                    out_syn_file.unions.push(union_item.clone());
                                    remain_applications.extend(union_item.get_applications());
                                }
                                ContextItem::Struct(struct_item) => {
                                    out_syn_file.structs.push(struct_item.clone());
                                    remain_applications.extend(struct_item.get_applications());
                                }
                                ContextItem::Trait(trait_item) => {
                                    out_syn_file.traits.push(trait_item.clone());
                                    remain_applications.extend(trait_item.get_applications());
                                }
                            }
                        }
                    }
                }
                let file_path = function_path.clone().join("context.rs");
                let mut file = fs::File::create(file_path).unwrap();
                file.write_all(out_syn_file.to_string().as_bytes()).unwrap();
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct SynFile {
    pub file_name: String,
    pub file_path: PathBuf,
    pub consts: Vec<ConstItem>,
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
            file_path: PathBuf::new(),
            consts: Vec::new(),
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
    pub fn new_with_file_path(file_path: PathBuf) -> Self {
        let mut syn_file = SynFile::new();
        syn_file.file_path = file_path.clone();
        syn_file.file_name = file_path
            .clone()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        syn_file
    }
    pub fn from_syntax(file_path: PathBuf, syntax: &File) -> Self {
        let mut syn_file = SynFile::new_with_file_path(file_path);
        for item in syntax.items.clone() {
            match item {
                Item::Const(item_const) => {
                    let mut const_item = ConstItem::new();
                    const_item.item = Some(item_const);
                    syn_file.consts.push(const_item);
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
                    mod_item.item = Some(item_mod.clone());
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
                    let mut empty_item_impl = item_impl.clone();
                    empty_item_impl.items = Vec::new();
                    impl_item.item = Some(empty_item_impl);
                    for item in item_impl.items.iter() {
                        match item {
                            SynImplItem::Const(const_item) => {
                                impl_item.consts.push(const_item.clone());
                            }
                            SynImplItem::Type(type_item) => {
                                impl_item.types.push(type_item.clone());
                            }
                            SynImplItem::Fn(fn_item) => {
                                let mut function_item = FunctionItem::new();
                                function_item.function_name = fn_item.sig.ident.to_string();
                                function_item.item = Some(MyItemFn::ImplFn(fn_item.clone()));
                                function_item.applications =
                                    visit_stmts(fn_item.block.stmts.clone());
                                impl_item.functions.push(function_item);
                            }
                            _ => {}
                        }
                    }
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
                    function_item.item = Some(MyItemFn::Fn(item_fn.clone()));
                    function_item.applications = visit_stmts(item_fn.block.stmts.clone());
                    syn_file.functions.push(function_item);
                }
                Item::Trait(item_trait) => {
                    let mut trait_item = TraitItem::new();
                    let trait_name: String = item_trait.ident.to_string();
                    trait_item.trait_name = trait_name;
                    let mut empty_item_trait = item_trait.clone();
                    empty_item_trait.items = Vec::new();
                    trait_item.item = Some(empty_item_trait);
                    for item in item_trait.items.iter() {
                        match item {
                            SynTraitItem::Const(const_item) => {
                                trait_item.consts.push(const_item.clone());
                            }
                            SynTraitItem::Type(type_item) => {
                                trait_item.types.push(type_item.clone());
                            }
                            SynTraitItem::Fn(fn_item) => {
                                let mut function_item = FunctionItem::new();
                                function_item.function_name = fn_item.sig.ident.to_string();
                                function_item.item = Some(MyItemFn::TraitFn(fn_item.clone()));
                                if let Some(block) = &fn_item.default {
                                    function_item.applications = visit_stmts(block.stmts.clone());
                                }
                                trait_item.functions.push(function_item);
                            }
                            _ => {}
                        }
                    }
                    syn_file.traits.push(trait_item);
                }
                _ => {}
            }
        }
        syn_file
    }
    pub fn change_applications(&mut self, all_names: &Vec<String>) {
        for function_item in self.functions.iter_mut() {
            function_item
                .applications
                .retain(|application| all_names.contains(application));
        }
        for struct_item in self.structs.iter_mut() {
            for impl_item in struct_item.impls.iter_mut() {
                for function_item in impl_item.functions.iter_mut() {
                    function_item
                        .applications
                        .retain(|application| all_names.contains(application));
                    function_item.applications.extend(
                        struct_item
                            .traits
                            .iter()
                            .map(|trait_name| trait_name.clone()),
                    );
                }
            }
            for trait_impl_item in struct_item.traits_impls.iter_mut() {
                for function_item in trait_impl_item.functions.iter_mut() {
                    function_item
                        .applications
                        .retain(|application| all_names.contains(application));
                    function_item.applications.extend(
                        struct_item
                            .traits
                            .iter()
                            .map(|trait_name| trait_name.clone()),
                    );
                }
            }
        }
        for enum_item in self.enums.iter_mut() {
            for impl_item in enum_item.impls.iter_mut() {
                for function_item in impl_item.functions.iter_mut() {
                    function_item
                        .applications
                        .retain(|application| all_names.contains(application));
                    function_item
                        .applications
                        .extend(enum_item.traits.iter().map(|trait_name| trait_name.clone()));
                }
            }
            for trait_impl_item in enum_item.traits_impls.iter_mut() {
                for function_item in trait_impl_item.functions.iter_mut() {
                    function_item
                        .applications
                        .retain(|application| all_names.contains(application));
                    function_item
                        .applications
                        .extend(enum_item.traits.iter().map(|trait_name| trait_name.clone()));
                }
            }
        }
        for union_item in self.unions.iter_mut() {
            for impl_item in union_item.impls.iter_mut() {
                for function_item in impl_item.functions.iter_mut() {
                    function_item
                        .applications
                        .retain(|application| all_names.contains(application));
                    function_item.applications.extend(
                        union_item
                            .traits
                            .iter()
                            .map(|trait_name| trait_name.clone()),
                    );
                }
            }
            for trait_impl_item in union_item.traits_impls.iter_mut() {
                for function_item in trait_impl_item.functions.iter_mut() {
                    function_item
                        .applications
                        .retain(|application| all_names.contains(application));
                    function_item.applications.extend(
                        union_item
                            .traits
                            .iter()
                            .map(|trait_name| trait_name.clone()),
                    );
                }
            }
        }
        for trait_item in self.traits.iter_mut() {
            for function_item in trait_item.functions.iter_mut() {
                function_item
                    .applications
                    .retain(|application| all_names.contains(application));
            }
        }
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
                .map(|trait_item| trait_item.to_item_trait()),
        );
        for struct_item in &self.structs {
            items.push(Item::Struct(struct_item.item.clone().unwrap()));
            items.extend(
                struct_item
                    .impls
                    .iter()
                    .map(|impl_item| impl_item.to_item_impl()),
            );
            items.extend(
                struct_item
                    .traits_impls
                    .iter()
                    .map(|trait_impl_item| trait_impl_item.to_item_impl()),
            );
        }
        for enum_item in &self.enums {
            items.push(Item::Enum(enum_item.item.clone().unwrap()));
            items.extend(
                enum_item
                    .impls
                    .iter()
                    .map(|impl_item| impl_item.to_item_impl()),
            );
            items.extend(
                enum_item
                    .traits_impls
                    .iter()
                    .map(|trait_impl_item| trait_impl_item.to_item_impl()),
            );
        }
        for union_item in &self.unions {
            items.push(Item::Union(union_item.item.clone().unwrap()));
            items.extend(
                union_item
                    .impls
                    .iter()
                    .map(|impl_item| impl_item.to_item_impl()),
            );
            items.extend(
                union_item
                    .traits_impls
                    .iter()
                    .map(|trait_impl_item| trait_impl_item.to_item_impl()),
            );
        }
        let mut functions: Vec<ItemFn> = Vec::new();
        for function in self.functions.iter() {
            if let MyItemFn::Fn(item_fn) = function.item.clone().unwrap() {
                functions.push(item_fn);
            }
        }
        items.extend(
            functions
                .iter()
                .map(|function_item| Item::Fn(function_item.clone())),
        );
        let tokens = quote! { # (# items) * };
        tokens.to_string()
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
impl SEUGetApplications for UnionItem {
    fn get_impls(&self) -> Vec<ImplItem> {
        self.impls.clone()
    }
    fn get_traits_impls(&self) -> Vec<ImplItem> {
        self.traits_impls.clone()
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
pub struct TraitItem {
    trait_name: String,
    item: Option<ItemTrait>,
    types: Vec<TraitItemType>,
    consts: Vec<TraitItemConst>,
    functions: Vec<FunctionItem>,
}
impl TraitItem {
    fn new() -> Self {
        TraitItem {
            trait_name: String::new(),
            item: None,
            types: Vec::new(),
            consts: Vec::new(),
            functions: Vec::new(),
        }
    }
    fn to_item_trait(&self) -> Item {
        let mut item_impl = self.item.clone().unwrap();
        item_impl.items.extend(
            self.types
                .iter()
                .map(|type_item| SynTraitItem::Type(type_item.clone())),
        );
        item_impl.items.extend(
            self.consts
                .iter()
                .map(|const_impl| SynTraitItem::Const(const_impl.clone())),
        );
        let mut functions: Vec<SynTraitItem> = Vec::new();
        for function in self.functions.iter() {
            if let MyItemFn::TraitFn(item_function) = function.item.clone().unwrap() {
                functions.push(SynTraitItem::Fn(item_function));
            }
        }
        item_impl.items.extend(functions);
        Item::Trait(item_impl)
    }
    fn get_applications(&self) -> Vec<String> {
        let mut applications: Vec<String> = Vec::new();
        for function_item in self.functions.iter() {
            applications.extend(function_item.applications.clone());
        }
        applications
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
impl SEUGetApplications for StructItem {
    fn get_impls(&self) -> Vec<ImplItem> {
        self.impls.clone()
    }
    fn get_traits_impls(&self) -> Vec<ImplItem> {
        self.traits_impls.clone()
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
pub struct ModItem {
    item: Option<ItemMod>,
}
impl ModItem {
    fn new() -> Self {
        ModItem { item: None }
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
pub struct ImplItem {
    pub item: Option<ItemImpl>,
    pub types: Vec<ImplItemType>,
    pub consts: Vec<ImplItemConst>,
    pub functions: Vec<FunctionItem>,
}
impl ImplItem {
    fn new() -> Self {
        ImplItem {
            item: None,
            types: Vec::new(),
            consts: Vec::new(),
            functions: Vec::new(),
        }
    }
    fn to_item_impl(&self) -> Item {
        let mut item_impl = self.item.clone().unwrap();
        item_impl.items.extend(
            self.types
                .iter()
                .map(|type_item| SynImplItem::Type(type_item.clone())),
        );
        item_impl.items.extend(
            self.consts
                .iter()
                .map(|const_impl| SynImplItem::Const(const_impl.clone())),
        );
        let mut functions: Vec<SynImplItem> = Vec::new();
        for function in self.functions.iter() {
            if let MyItemFn::ImplFn(item_function) = function.item.clone().unwrap() {
                functions.push(SynImplItem::Fn(item_function));
            }
        }
        item_impl.items.extend(functions);
        Item::Impl(item_impl)
    }
}
#[derive(Debug, Clone)]
pub struct FunctionItem {
    function_name: String,
    item: Option<MyItemFn>,
    applications: Vec<String>,
}
impl FunctionItem {
    fn new() -> Self {
        FunctionItem {
            function_name: String::new(),
            item: None,
            applications: Vec::new(),
        }
    }
    fn get_applications(&self) -> Vec<String> {
        self.applications.clone()
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
impl SEUGetApplications for EnumItem {
    fn get_impls(&self) -> Vec<ImplItem> {
        self.impls.clone()
    }
    fn get_traits_impls(&self) -> Vec<ImplItem> {
        self.traits_impls.clone()
    }
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
#[derive(Parser)]
#[command(name = "rust focxt")]
#[command(author = "AbeZbm")]
#[command(version = "1.0")]
# [command (about = "A rust program to get focal context." , long_about = None)]
struct Cli {
    #[doc = "Sets project path"]
    #[arg(short = 'p', long = "project")]
    project: Option<String>,
}
#[derive(Debug, Clone)]
enum MyItemFn {
    Fn(ItemFn),
    ImplFn(ImplItemFn),
    TraitFn(TraitItemFn),
}
enum ContextItem {
    Struct(StructItem),
    Enum(EnumItem),
    Union(UnionItem),
    Trait(TraitItem),
}
fn main() {
    let cli = Cli::parse();
    let project_path = fs::canonicalize(PathBuf::from(cli.project.unwrap())).unwrap();
    let rs_files: Vec<PathBuf> = find_rs_files(&project_path);
    let mut syn_files = SynFiles::new();
    for rs_file in rs_files {
        let code = read_to_string(&rs_file).unwrap();
        let syntax = parse_file(&code).unwrap();
        let syn_file = SynFile::from_syntax(rs_file.clone(), &syntax);
        syn_files.add_syn_file(syn_file);
    }
    syn_files.get_all_names();
    syn_files.change_applications();
    syn_files.get_all_context(project_path);
}
