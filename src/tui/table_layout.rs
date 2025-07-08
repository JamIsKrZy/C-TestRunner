use core::fmt;
use std::{ cell::{Cell, RefCell}, error::Error, io::{Stdout, Write}, usize};

use termion::{color, raw::RawTerminal, style};

use std::fmt::Write as FmtWrite;

use crate::tui::{OriginPoint, TerminalSize};




#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum GapMode{
    Fixed(usize),
    LeftSpace,
}

pub(super) enum TableHeader<S: AsRef<str>,const N: usize>{
    ColumnTitle([String; N]),
    TableTitle(S)
}


pub(super) struct TableLayout<'a, const N: usize>{
    width_perc: f32,
    height_perc: f32,

    column_gaps: Option<[GapMode; N]>,
    title_column: Option<TableHeader<&'a str, N>>
}



pub(super) struct TableDisplay<'a,'b, const N: usize>{
    term_size: &'a TerminalSize,
    origin: OriginPoint,

    dimension: (u16, u16),
    computed_gaps: [usize; N],

    width_perc: f32,
    height_perc: f32,

    cell_gap: [GapMode; N],
    title: RefCell<TableHeader<&'b str, N>>
}



impl<'a, const N: usize> TableLayout<'a, N>{

    pub fn new(width_perc: f32, height_perc: f32) -> Self {
        Self { 
            width_perc, 
            height_perc,  
            column_gaps: None, 
            title_column: None 
        }
    }

    #[inline]
    fn default_gaps() -> GapMode {
        GapMode::LeftSpace
    }

    #[inline]
    fn default_title() -> &'static str {
        "Unamed"
    } 


    pub fn set_column_gaps(mut self, gaps: [GapMode; N]) -> Self {
        if gaps.len() != N.into() {
            panic!("Provided column title does not have the same ammount as the column");
        }

        self.column_gaps = Some(gaps);
        self
    }

    pub fn set_column_title(mut self, titles: [String; N])-> Self{ 
        self.title_column = Some(TableHeader::ColumnTitle(titles));
        self        
    }

    pub fn set_table_title(mut self, title: &'a str) -> Self {
        self.title_column = Some(TableHeader::TableTitle(title));
        self
    }

    pub fn build(self, origin: OriginPoint, term_size: &'a TerminalSize) -> TableDisplay<'_, '_, N> {

        let cell_gap = self.column_gaps.unwrap_or({
            let gap = Self::default_gaps();
            [gap; N]
        });

        let dimension = TableDisplay::<N>::compute_dimension(
            &term_size, 
            self.width_perc, 
            self.height_perc
        );

        let table_style = self.title_column.unwrap_or({
                TableHeader::TableTitle("Untitled")   
        });


        let computed_gaps = TableDisplay::compute_gaps(
            dimension,
            &cell_gap,
            &table_style
        ); 

        TableDisplay { 
            term_size, 
            origin, 
            dimension, 
            computed_gaps,
            width_perc: self.width_perc, 
            height_perc: self.height_perc, 
            cell_gap,
            title: RefCell::new(table_style)
        }
    }

}


impl<'a, 'b, const N: usize> TableDisplay<'a, 'b, N>{

    fn compute_dimension(
        term: &TerminalSize, 
        width_perc: f32,
        height_perc: f32
    ) -> (u16,u16) {        
        let term_dim= term.read();
        (
            (term_dim.0 as f32 * (width_perc / 100.0)).round() as u16,
            (term_dim.1 as f32 * (height_perc / 100.0)).round() as u16
        )
    }

    pub fn set_header(&self, table_stye: TableHeader<&'b str, N>){
        *self.title.borrow_mut() = table_stye;
    }

    pub fn compute_gaps( 
        dim: (u16,u16), 
        gap_list: &[GapMode; N],
        table_stye: &TableHeader<&'b str, N>
    ) -> [usize;N]{
        let mut width_space: usize = dim.0
            .saturating_sub(2)
            .into();

        let mut spaces = [0usize;N];
        
        let mut count = 0;

        gap_list.iter().enumerate().for_each(|(i, g)|{
            match g {
                GapMode::Fixed(s) => {
                    width_space -= *s;
                    spaces[i] = *s;
                },
                GapMode::LeftSpace => {
                    count += 1;
                },
            }
        });

        if count != 0{
            let share_space = width_space / count;

            gap_list.iter().enumerate().for_each(|(i, g)|{
                if g == &GapMode::LeftSpace{
                    spaces[i] = share_space;
                }
            });    
        }

        if matches!(table_stye, TableHeader::ColumnTitle(_)) {
            let last_index = spaces.len();

            spaces.iter_mut().skip(last_index).for_each(|i|{
                *i += 1;
            });
        }
        
        spaces
    }

    pub fn get_window_fit(&self) -> usize {
        self.dimension
            .1
            .saturating_sub(4)
            .into()
    }
   
    

    fn format_top(&self, fmt: &mut String) -> fmt::Result {
        write!(fmt, "┌")?;

        match &*self.title.borrow() {
            TableHeader::ColumnTitle(_) => {
                let gaps = self.computed_gaps
                    .as_ref();

                for &g in &gaps[..gaps.len() - 1]{
                    write!(fmt, "{}┬",
                        "─".repeat((g - 1).into()),
                    )?;
                }

                if let Some(&g) = gaps.last(){
                    write!(fmt, "{}",
                        "─".repeat(g.into()),
                    )?;
                }
            },
            TableHeader::TableTitle(_) => {
                let gap = self.dimension
                    .0
                    .saturating_sub(2);

                write!(fmt, "{}",
                    "─".repeat(gap.into())
                )?;
            },
        }
            
        writeln!(fmt, "┐\r")
    }


    fn format_bottom(&self, fmt: &mut String) -> fmt::Result {
        write!(fmt, "└")?;

        let gap = self.dimension
            .0
            .saturating_sub(2);

        write!(fmt, "{}",
            "─".repeat(gap.into())
        )?;
            
        writeln!(fmt, "┘\r")
    }  

    fn format_table_title(&self, fmt: &mut String) -> fmt::Result{
        write!(fmt, "│{}",
            style::Bold
        )?;

        match &*self.title.borrow() {
            TableHeader::ColumnTitle(title_list) => {
                let gaps = self.computed_gaps
                    .as_ref();

                for (&g, s) in (&gaps[..gaps.len()-1]).iter().zip(title_list.iter()){
                   write!(fmt, "{:<gap_size$}│",
                        s,
                        gap_size = (g - 1).into(),
                    )?;
                }

                if let Some((&g, s)) = gaps.last().zip(title_list.last()){
                    write!(fmt, "{:<gap_size$}",
                        s,
                        gap_size = g.into(),
                    )?;
                }
            },
            TableHeader::TableTitle(head) => {
                let width = self.dimension.0;
                    
                let gap = (width - 4).into();
                write!(fmt, "  {:<gap$}", head)?;
            },
        }

        writeln!(fmt, "{}│\r",
            style::Reset
        )
    }

    fn format_colum_title_bottom(&self, fmt: &mut String) -> fmt::Result {
        write!(fmt, "├")?;

        match &*self.title.borrow() {
            TableHeader::ColumnTitle(_) => {
                let gaps = self.computed_gaps
                    .as_ref();

                for &gap in (&gaps[..gaps.len()-1]).iter(){
                   write!(fmt, "{}┴",
                        "─".repeat((gap - 1).into())
                    )?;
                }

                let &gap = gaps.last().expect("No last gap");
                write!(fmt, "{}",
                    "─".repeat(gap.into())
                )?;
                
            },
            TableHeader::TableTitle(_) => {
                let width = self.dimension.0;

                let gap = width - 2;
                write!(fmt, "{}", "─".repeat(gap.into()))?;
            },
        }
        
            
        writeln!(fmt, "┤\r")
    }

    fn format_highlight_item<R>(
        &self, 
        fmt: &mut String, 
        content: &[R; N]
    ) -> fmt::Result 
        where R: AsRef<str>
    {
        write!(fmt, "│{}",
            color::Bg(color::Blue)
        )?;

        let gaps = self.computed_gaps
            .as_ref();

        for (&g, s) in gaps.iter().zip(content.iter()){
            write!(fmt, "{:<gap_size$}",
                s.as_ref(),
                gap_size = g.into(),
            )?;
        }

        

        writeln!(fmt, "{}│\r",
            color::Bg(color::Reset)
        )
    }


    fn format_item<R>(
        &self, 
        fmt: &mut String, 
        content: &[R; N]
    ) -> fmt::Result 
        where R: AsRef<str>
    {
        write!(fmt, "│",)?;


        let gaps = self.computed_gaps
            .as_ref();

        for (&g, s) in gaps.iter().zip(content.iter()){
            write!(fmt, "{:<gap_size$}",
                s.as_ref(),
                gap_size = g.into(),
            )?;
        }
            
        writeln!(fmt, "│\r",)
    }




// | Type         | Character | Unicode | Meaning                     |
// | ------------ | --------- | ------- | --------------------------- |
// | Horizontal   | `─`       | U+2500  | Horizontal line             |
// | Vertical     | `│`       | U+2502  | Vertical line               |
// | Top-left     | `┌`       | U+250C  | Top-left corner             |
// | Top-right    | `┐`       | U+2510  | Top-right corner            |
// | Bottom-left  | `└`       | U+2514  | Bottom-left corner          |
// | Bottom-right | `┘`       | U+2518  | Bottom-right corner         |
// | T Down       | `┬`       | U+252C  | T-connection pointing down  |
// | T Up         | `┴`       | U+2534  | T-connection pointing up    |
// | T Left       | `┤`       | U+2524  | T-connection pointing left  |
// | T Right      | `├`       | U+251C  | T-connection pointing right |
// | Cross        | `┼`       | U+253C  | Cross intersection          |



    pub fn refresh(&self) -> fmt::Result {
        todo!()
    }

    pub fn write<R>(
        &self, 
        stdout: &mut RawTerminal<Stdout>, 
        contents: &[[R; N]], 
        highlight_index: usize
    ) -> Result<(), Box<dyn Error>> 
        where R: AsRef<str>
    {
        let mut buffer = String::new();


        //print header
        self.format_top(&mut buffer)?;
        self.format_table_title(&mut buffer)?;
        self.format_colum_title_bottom(&mut buffer)?;

        //print content
        for (i,content) in contents.into_iter().enumerate(){
            if i == highlight_index {
                self.format_highlight_item(&mut buffer, &content)?;
            } else {
                self.format_item(&mut buffer, content)?;
            }
        }
        
        // print bottom
        self.format_bottom(&mut buffer)?;


        write!(stdout, "{}", buffer)?;

        Ok(())
    }
    
    
}

