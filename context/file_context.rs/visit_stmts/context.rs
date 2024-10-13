struct PathVisitor {
    paths: Vec<String>,
}
impl<'ast> Visit<'ast> for PathVisitor {
    fn visit_path(&mut self, node: &'ast Path) {
        self.paths.extend(
            node.segments
                .iter()
                .map(|segment| segment.ident.to_string()),
        );
        visit::visit_path(self, node)
    }
}
fn visit_stmts(stmts: Vec<Stmt>) -> Vec<String> {
    let mut applications: Vec<String> = Vec::new();
    let mut visitor = PathVisitor { paths: Vec::new() };
    for stmt in stmts {
        visitor.visit_stmt(&stmt);
    }
    applications.extend(visitor.paths.iter().map(|path| path.clone()));
    applications.sort();
    applications.dedup();
    applications
}
