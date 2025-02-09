use rustc_driver::Compilation;
use rustc_hir::def;
use rustc_hir::ExprKind;
use rustc_hir::StmtKind;
use rustc_interface::interface;
use rustc_interface::Queries;
use rustc_middle::ty::TyCtxt;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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
        let hir_krate = tcx.hir();
        for id in hir_krate.items() {
            let item = id.owner_id.def_id;
            match tcx.def_kind(item) {
                def::DefKind::Fn | def::DefKind::AssocFn => {
                    // println!("{:#?}", item);
                    let fn_name = format!("{:?}", item.to_def_id());
                    let hir = hir_krate.body_owned_by(item);
                    let mir = tcx.mir_built(item).borrow();
                    // println!("{:#?}", mir);
                    let directory_path = "./hir";
                    create_dir_all(&directory_path).unwrap();
                    let file_path = PathBuf::from(&directory_path).join(format!("{}.txt", fn_name));
                    let mut file = File::create(&file_path).unwrap();
                    file.write_all(format!("{:#?}", hir).as_bytes()).unwrap();
                    let directory_path = "./mir";
                    create_dir_all(&directory_path).unwrap();
                    let file_path = PathBuf::from(&directory_path).join(format!("{}.txt", fn_name));
                    let mut file = File::create(&file_path).unwrap();
                    file.write_all(format!("{:#?}", mir).as_bytes()).unwrap();
                }
                _ => {
                    println!("mir other kind: {:?}", tcx.def_kind(item));
                }
            }
        }
    }
}
