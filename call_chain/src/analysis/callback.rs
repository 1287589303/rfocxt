use regex::Regex;
use rustc_driver::Compilation;
use rustc_hir::def;
use rustc_interface::interface;
use rustc_interface::Queries;
use rustc_middle::mir::Operand;
use rustc_middle::mir::TerminatorKind;
use rustc_middle::ty;
use rustc_middle::ty::TyCtxt;
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::hirvisitor::HirVisitor;
use super::hirvisitor::VisitorData;

pub struct MirCheckerCallbacks {
    pub source_name: String,
    // cond_map: HashMap<SourceInfo, Condition>,
}

impl MirCheckerCallbacks {
    pub fn new() -> Self {
        Self {
            source_name: String::new(),
            // cond_map: HashMap::new(),
        }
    }
}

impl rustc_driver::Callbacks for MirCheckerCallbacks {
    /// Called before creating the compiler instance
    fn config(&mut self, config: &mut interface::Config) {
        self.source_name = format!("{:?}", config.input.source_name());
        config.crate_cfg.push("mir_checker".to_string());
        info!("Source file: {}", self.source_name);
    }

    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        _queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        _queries
            .global_ctxt()
            .unwrap()
            .enter(|tcx| self.run_analysis(tcx));

        Compilation::Continue
    }

    // fn after_analysis<'tcx>(
    //     &mut self,
    //     _compiler: &interface::Compiler,
    //     _queries: &'tcx Queries<'tcx>,
    // ) -> Compilation {
    //     _queries
    //         .global_ctxt()
    //         .unwrap()
    //         .enter(|tcx| self.run_analysis(tcx, 1));

    //     Compilation::Continue
    // }
}

impl MirCheckerCallbacks {
    fn run_analysis<'tcx, 'compiler>(&mut self, tcx: TyCtxt<'tcx>) {
        // let hir_krate = tcx.hir();
        // for id in hir_krate.items() {
        //     let item = id.owner_id.def_id;
        //     match tcx.def_kind(item) {
        //         def::DefKind::Fn | def::DefKind::AssocFn => {
        //             // println!("{:#?}", item);
        //             let fn_name = format!("{:?}", item.to_def_id());
        //             let hir = hir_krate.body_owned_by(item);
        //             let mir = tcx.mir_built(item).borrow();
        //             // println!("{:#?}", mir);
        //             let directory_path = "./hir";
        //             create_dir_all(&directory_path).unwrap();
        //             let file_path = PathBuf::from(&directory_path).join(format!("{}.txt", fn_name));
        //             let mut file = File::create(&file_path).unwrap();
        //             file.write_all(format!("{:#?}", hir).as_bytes()).unwrap();
        //             let directory_path = "./mir";
        //             create_dir_all(&directory_path).unwrap();
        //             let file_path = PathBuf::from(&directory_path).join(format!("{}.txt", fn_name));
        //             let mut file = File::create(&file_path).unwrap();
        //             file.write_all(format!("{:#?}", mir).as_bytes()).unwrap();
        //         }
        //         _ => {
        //             println!("mir other kind: {:?}", tcx.def_kind(item));
        //         }
        //     }
        // }
        let hir_map = tcx.hir();
        let mut visitor = HirVisitor::new(tcx, hir_map);
        // hir_map.visit_all_item_likes_in_crate(&mut visitor);
        hir_map.walk_toplevel_module(&mut visitor);
        let result = visitor.move_result();
        for data in result {
            let VisitorData {
                id,
                fn_name,
                doc,
                has_ret,
                mod_info,
                visible,
                fn_source,
                basic_blocks,
                local_decls,
            } = data;
            let mut calls: HashSet<String> = HashSet::new();
            let mut types: HashSet<String> = HashSet::new();
            for basic_block in basic_blocks {
                if let TerminatorKind::Call {
                    func,
                    args,
                    destination,
                    target,
                    unwind,
                    call_source,
                    fn_span,
                } = &basic_block.terminator().kind
                {
                    // println!("{:#?}", &basic_block.terminator().kind);
                    let kind_string = format!("{:#?}", &basic_block.terminator().kind);
                    let kind_strings: Vec<&str> = kind_string.splitn(3, ' ').into_iter().collect();
                    let kind_string = kind_strings[2];
                    let call_string = &kind_string[..kind_string.find("(").unwrap()];
                    // println!("提取的函数调用：{}", call_string);
                    calls.insert(call_string.to_string());
                }
            }
            println!("Calls:");
            for call in calls.iter() {
                println!("{:#?}", call);
            }
            for local_decl in local_decls {
                let decl_type = local_decl.ty.peel_refs().to_string();
                // println!("{:#?}", local_decl.ty.peel_refs().to_string());
                types.insert(decl_type);
            }
            println!("Types:");
            for a_type in types.iter() {
                println!("{:#?}", a_type);
            }
            println!();
            // let directory_path = "./hir";
            // create_dir_all(&directory_path).unwrap();
            // let file_path = PathBuf::from(&directory_path).join(format!("{}.txt", fn_name));
            // let mut file = File::create(&file_path).unwrap();
            // file.write_all(format!("{:#?}", hir).as_bytes()).unwrap();
        }
    }
}
