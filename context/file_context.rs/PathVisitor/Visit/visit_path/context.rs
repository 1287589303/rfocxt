struct PathVisitor { paths : Vec < String > , } impl < 'ast > Visit < 'ast > for PathVisitor { fn visit_path (& mut self , node : & 'ast Path) { self . paths . extend (node . segments . iter () . map (| segment | segment . ident . to_string ()) ,) ; visit :: visit_path (self , node) } }