use regex::Regex;
use rustc_driver::Compilation;
use rustc_hir::def;
use rustc_interface::interface;
use rustc_interface::Queries;
use rustc_middle::mir::Operand;
use rustc_middle::mir::TerminatorKind;
use rustc_middle::ty;
use rustc_middle::ty::GenericArgKind;
use rustc_middle::ty::Ty;
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::TyKind;
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::exporter::CallsAndTypes;
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

fn collect_subtypes<'tcx>(ty: Ty<'tcx>, tcx: TyCtxt<'tcx>, result: &mut HashSet<Ty<'tcx>>) {
    let ty = ty.peel_refs();
    if !result.insert(ty) {
        return;
    }
    result.insert(ty);

    match ty.kind() {
        // 处理结构体/枚举类型（ADT）
        TyKind::Adt(adt, args) => {
            // if !adt.generics().params.is_empty() {
            //     let generic_ty =
            //         tcx.mk_adt(
            //             *adt,
            //             tcx.mk_args_from_iter(adt.generics().params.iter().map(|param| {
            //                 GenericArg::from(tcx.mk_ty_param(param.index, param.name))
            //             })),
            //         );
            //     collect_subtypes(generic_ty, tcx, result);
            // }
            for arg in args.iter() {
                if let GenericArgKind::Type(sub_ty) = arg.unpack() {
                    collect_subtypes(sub_ty, tcx, result);
                }
            }
        }

        // 处理数组类型 [T; N]
        TyKind::Array(sub_ty, _) => {
            collect_subtypes(*sub_ty, tcx, result);
        }

        // 处理切片类型 [T]
        TyKind::Slice(sub_ty) => {
            collect_subtypes(*sub_ty, tcx, result);
        }

        // 处理原始指针类型 *const T/*mut T
        TyKind::RawPtr(ty_mut, _) => {
            collect_subtypes(*ty_mut, tcx, result);
        }

        // 处理元组类型 (T1, T2, ...)
        TyKind::Tuple(sub_tys) => {
            for sub_ty in sub_tys.iter() {
                collect_subtypes(sub_ty, tcx, result);
            }
        }
        // 处理其他类型...
        _ => {}
    }
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
            let mut tys: HashSet<Ty<'tcx>> = HashSet::new();
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

                    for arg in args.iter() {
                        if let Operand::Constant(constant) = &arg.node {
                            // let arg_type = constant.ty().peel_refs().to_string();
                            // types.insert(arg_type);
                            collect_subtypes(constant.ty(), tcx, &mut tys);
                        }
                    }
                }
            }
            // println!("{}", fn_name);
            // println!("Calls:");
            // for call in calls.iter() {
            //     println!("{:#?}", call);
            // }
            // for local_decl in local_decls {
            //     // let decl_type = local_decl.ty.peel_refs().to_string();
            //     // println!("{:#?}", local_decl.ty.peel_refs().to_string());
            //     // types.insert(decl_type);
            //     collect_subtypes(local_decl.ty, tcx, &mut tys);
            // }
            // for ty in tys.iter() {
            //     types.insert(ty.to_string());
            // }
            // println!("Types:");
            // for a_type in types.iter() {
            //     println!("{:#?}", a_type);
            // }
            // println!();
            let calls_and_types = CallsAndTypes::new(&calls, &types);
            let directory_path = "./rfocxt/callsandtypes";
            create_dir_all(&directory_path).unwrap();
            let file_path = PathBuf::from(&directory_path).join(format!("{}.json", fn_name));
            let mut file = File::create(&file_path).unwrap();
            file.write_all(serde_json::to_string(&calls_and_types).unwrap().as_bytes())
                .unwrap();
        }
    }
}
