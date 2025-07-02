use std::{cell::RefCell, io::Stdout};



mod test;
mod table_layout;

struct TerminalSize(RefCell<(u16,u16)>);
struct OriginPoint(u16,u16);

pub fn display_collection(){
    


}


trait RenderComponent{

    fn refresh(&self, stdout: Stdout);
    fn render(&self, stdout: Stdout);
}