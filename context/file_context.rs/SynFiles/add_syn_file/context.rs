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