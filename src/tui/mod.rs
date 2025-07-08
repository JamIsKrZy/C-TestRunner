use core::fmt;
use std::{cell::{Ref, RefCell}, fmt::Formatter, io::{self, stdin, stdout, Stdout, Write}};

use nix::libc::option;
use termion::{cursor, input::TermRead, raw::{IntoRawMode, RawTerminal}};
use termion::event::Key;

use crate::{record_collection::collection::CompiledRecord, tui::table_layout::{GapMode, TableDisplay, TableHeader, TableLayout}};




mod test;
mod table_layout;

pub use test::sample_table;
pub use test::init_test_collection;

#[derive(Debug)]
struct TerminalSize(RefCell<(u16,u16)>);

#[derive(Debug, Clone, Copy)]
struct OriginPoint(u16,u16);


#[repr(usize)]
#[derive(Debug, PartialEq, Clone, Copy)]
enum StateDisplay{
    ProgramList = 0,
    TestList = 1
}



fn init_tables(term: &TerminalSize) -> Box<[TableDisplay<2>; 2]>{
    let table_origin = OriginPoint(0,0);

    let program_table = TableLayout::new(30.0, 90.0)
        .set_column_gaps([GapMode::LeftSpace, GapMode::Fixed(10)])
        .set_column_title(["Test Programs".to_string(), "  Result".to_string()])
        .build(table_origin, &term);

    let test_table = TableLayout::new(30.0, 90.0)
        .set_table_title("[Unassigned]")
        .set_column_gaps([GapMode::LeftSpace, GapMode::Fixed(10)])
        .build(table_origin, &term);

    Box::new([program_table, test_table])
}

pub fn display_collection(collection: CompiledRecord){
    let (_owned_string, 
        summary_program, 
        program_details) = collection.summarize();

    
    let term = TerminalSize::new();
    let tables = init_tables(&term);
    
    let mut previous_state = None;
    let mut visiting_program = 0usize;

    let mut window_state = 0usize;
    let mut display_state= StateDisplay::ProgramList;
    let mut selection_i = 0usize;

    // TODO: upon refresh this will update
    let window_fit = tables[display_state as usize].get_window_fit();
    let mut selection_max_boundary = window_fit.min(summary_program.len()) - 1; 


    



    let stdin = stdin();
    let mut stdout = stdout()
        .into_raw_mode()
        .expect("unable to ");


    {
        write!(stdout, "{}{}{}",
            termion::clear::All,
            cursor::Goto(1,1),
            cursor::Hide
        ).unwrap();

        let (s, e) = {
            (window_state,(window_state + window_fit).min(summary_program.len()))
        };

        tables[display_state as usize].write(
            &mut stdout, 
            &summary_program[s..e], 
            selection_i
        );
    }

    for key in stdin.keys(){

        match key.expect("Key input Problem"){
            Key::Char('r') => {
                term.refresh();
                tables.iter().for_each(|t| {t.refresh();});
            },
            Key::Char('q') => {
                break;
            },
            Key::Up => {
                selection_i = selection_i.saturating_sub(1);
            },
            Key::Down => {
                selection_i = (selection_i+1)
                    .min(match display_state {
                        StateDisplay::ProgramList => summary_program.len(),
                        StateDisplay::TestList => program_details[visiting_program].len(),
                    } - 1);

                // TODO: add moving window 
                // if user goes beyond displayed list
                // more list to be displayed
                // through list.0
            },
            Key::Right | Key::Char('\n') => {
                if display_state == StateDisplay::ProgramList {
                    previous_state = Some((window_state, selection_i));
                    display_state = StateDisplay::TestList;

                    tables[1].set_header(TableHeader::TableTitle(
                        &summary_program[selection_i][0]
                    ));

                    visiting_program = selection_i;

                    selection_i = 0;
                    window_state = 0;
                }
            },
            Key::Left => {
                if let Some((ws, ss)) = previous_state {
                    window_state = ws;
                    selection_i = ss;

                    display_state = StateDisplay::ProgramList;
                    previous_state = None;
                }
            },
            _ => {}
        }


        write!(stdout, "{}{}",
            termion::clear::All,
            cursor::Goto(1,1)
        ).unwrap();

        let (s, e) = {
            (
                window_state,
                (window_state + window_fit)
                    .min(match display_state {
                        StateDisplay::ProgramList => summary_program.len(),
                        StateDisplay::TestList => program_details[visiting_program].len(),
                    })
            )

        };

        tables[display_state as usize].write(
            &mut stdout, 
            match display_state {
                StateDisplay::ProgramList => &summary_program[s..e],
                StateDisplay::TestList => &program_details[visiting_program][s..e],
            },
            selection_i
        );

        
        stdout.flush().unwrap();
    }


    write!(stdout, "{}",
        cursor::Show
    ).unwrap();
    


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
