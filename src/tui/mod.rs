use core::fmt;
use std::{cell::{Ref, RefCell}, fmt::Formatter, io::{self, stdin, stdout, Stdout}};

use termion::{input::TermRead, raw::{IntoRawMode, RawTerminal}};
use termion::event::Key;

use crate::{record_collection::collection::CompiledRecord, tui::table_layout::{GapMode, TableDisplay, TableLayout}};




mod test;
mod table_layout;

pub use test::sample_table;

struct TerminalSize(RefCell<(u16,u16)>);

#[derive(Debug, Clone, Copy)]
struct OriginPoint(u16,u16);


enum StateDisplay{
    ProgramList,
    TestList
}


fn init_tables(term: &TerminalSize) -> [TableDisplay<2>; 2]{
    let table_origin = OriginPoint(0,0);

    let program_table = TableLayout::new(70.0, 90.0)
        .set_column_gaps([GapMode::LeftSpace, GapMode::Fixed(10)])
        .set_column_title(["Test Programs".to_string(), "Result".to_string()])
        .build(table_origin, &term);

    let test_table = TableLayout::new(70.0, 90.0)
        .set_table_title("[Unassigned]".to_string())
        .set_column_gaps([GapMode::LeftSpace, GapMode::Fixed(10)])
        .build(table_origin, &term);

    [program_table, test_table]
}

pub fn display_collection(collection: CompiledRecord){
    let summary = collection.summarize();

    let stdin = stdin();
    let mut stdout = stdout()
        .into_raw_mode()
        .expect("unable to ");

    let term = TerminalSize::new();
    
    let tables = init_tables(&term);
    

    let state = (0usize, 0usize);
    let selection_i = 0usize;

    let window_fit = tables[state.1].get_window_fit();

    for key in stdin.keys(){

        let (s, e) = {
            (state.0,(state.0 + window_fit))
        };

        tables[state.1].write(
            &mut stdout, 
            &summary[s..e], 
            selection_i
        );

        match key.expect("Key input Problem"){
            Key::Char('r') => {
                term.refresh();
            },
            Key::Char('q') => {
                break;
            },
            Key::Up => {},
            Key::Down => {},
            Key::Right | Key::Char('\n') => {},
            Key::Left => {},
            _ => {}
        }
    }


}


trait RenderComponent{
    type T;

    fn refresh(&self) -> fmt::Result;
    fn write(&self, stdout: &mut RawTerminal<Stdout>, item: Vec<Self::T>) -> io::Result<()>;
}


impl TerminalSize{

    #[inline]
    fn get_terminal_size() -> (u16,u16){
        termion::terminal_size().unwrap_or_default()
    }
    
    pub fn new() -> Self {
        let dim = Self::get_terminal_size();
        Self(RefCell::new(dim))
    }

    pub fn refresh(&self){
        let dim = Self::get_terminal_size();
        *self.0.borrow_mut() = dim
    }

    pub fn read(&self) -> (u16,u16){
        *self.0.borrow()
    }
}
