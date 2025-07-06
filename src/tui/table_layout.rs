use core::fmt;
use std::{ error::Error, io::{self, Stdout, Write}, usize};

use termion::{color, raw::RawTerminal, style};

use std::fmt::Write as FmtWrite;

use crate::tui::{OriginPoint, RenderComponent, TerminalSize};


#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum GapMode{
    Fixed(usize),
    LeftSpace,
}

enum TableHeader<const N: usize>{
    ColumnTitle([String; N]),
    TableTitle(String)
}


pub(super) struct TableLayout<const N: usize>{
    width_perc: f32,
    height_perc: f32,

    column_gaps: Option<[GapMode; N]>,
    title_column: Option<TableHeader<N>>
}



pub(super) struct TableDisplay<'a, const N: usize>{
    term_size: &'a TerminalSize,
    origin: OriginPoint,

    dimension: Option<(u16, u16)>,
    computed_gaps: Option<[usize; N]>,

    width_perc: f32,
    height_perc: f32,

    cell_gap: [GapMode; N],
    title: TableHeader<N>
}



impl<const N: usize> TableLayout<N>{

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

    pub fn set_table_title(mut self, title: String) -> Self {
        self.title_column = Some(TableHeader::TableTitle(title));
        self
    }

    pub fn build<'a>(self, origin: OriginPoint, term_size: &'a TerminalSize) -> TableDisplay<'a, N> {
        TableDisplay { 
            term_size, 
            origin, 
            dimension: None, 
            computed_gaps: None,
            width_perc: self.width_perc, 
            height_perc: self.height_perc, 
            cell_gap: self.column_gaps.unwrap_or({
                let gap = Self::default_gaps();
                [gap; N]
            }),
            title: self.title_column.unwrap_or({
                TableHeader::TableTitle("Untitled".to_string())   
            }) 
        }
    }

}


impl<'a, const N: usize> TableDisplay<'_, N>{

    fn set_dimension(&mut self) -> (u16,u16) {        
        let dim = {
            let term_dim= self.term_size.read();
            (
                (term_dim.0 as f32 / (self.width_perc / 100.0)).round() as u16,
                (term_dim.0 as f32 / (self.width_perc / 100.0)).round() as u16
            )
        };

        self.dimension = Some(dim);
        dim
    }

    fn compute_gaps(&mut self){
        let mut overall_space: usize = self.dimension
            .expect("Dimension uninit")
            .0
            .saturating_sub(2)
            .into();

        let mut spaces = [0usize;N];


        let count_left = self.cell_gap.iter().filter(|&i|{
            i == &GapMode::LeftSpace            
        }).count();

        self.cell_gap.iter().enumerate().filter(|(_, g)|{
            matches!(g, GapMode::Fixed(_)) 
        }).map(|(i, g)|{
            if let GapMode::Fixed(s) = g{
                overall_space -= *s;
                spaces[i] = *s;
            }
        });

        let share_space = overall_space / count_left;

        self.cell_gap.iter().enumerate().filter(|(_, g)|{
            matches!(g, GapMode::LeftSpace) 
        }).map(|(i, g)|{
            if g == &GapMode::LeftSpace{
                spaces[i] = share_space;
            }
        });

        self.computed_gaps = Some(spaces);
    }

    pub fn get_window_fit(&self) -> usize {
        self.dimension
            .expect("Dimension uninit")
            .1
            .saturating_sub(4)
            .into()
    }
   
    

    fn format_top(&self, fmt: &mut String) -> fmt::Result {
        write!(fmt, "┌")?;

        match self.title {
            TableHeader::ColumnTitle(_) => {
                let gaps = self.computed_gaps
                    .as_ref()
                    .expect("Computed gaps is not initialized");

                for &g in &gaps[..gaps.len() - 1]{
                    write!(fmt, "{}┬",
                        "─".repeat(g.into()),
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
                    .expect("Dimenstion uninitiatialized")
                    .0
                    .saturating_sub(2);

                write!(fmt, "{}",
                    "─".repeat(gap.into())
                )?;
            },
        }
            
        writeln!(fmt, "┐")
    }


    fn format_bottom(&self, fmt: &mut String) -> fmt::Result {
        write!(fmt, "└")?;

        let gap = self.dimension
            .expect("Dimenstion uninitiatialized")
            .0
            .saturating_sub(2);

        write!(fmt, "{}",
            "─".repeat(gap.into())
        )?;
            
        writeln!(fmt, "┘")
    }  

    fn format_table_title(&self, fmt: &mut String) -> fmt::Result{
        write!(fmt, "│{}",
            style::Bold
        )?;

        match &self.title {
            TableHeader::ColumnTitle(title_list) => {
                let gaps = self.computed_gaps
                    .as_ref()
                    .expect("Computed gaps is not initialized");

                for (&g, s) in gaps.iter().zip(title_list.iter()){
                   write!(fmt, "{:<gap_size$}",
                        s,
                        gap_size = g.into(),
                    )?;
                }
            },
            TableHeader::TableTitle(head) => {
                let dim = self.dimension
                    .expect("Dimension not computed");
                    
                let gap = (dim.0 - 2).into();
                write!(fmt, "  {:<gap$}", head);
            },
        }

        write!(fmt, "{}│",
            style::Reset
        )
    }

    fn format_colum_title_bottom(&self, fmt: &mut String) -> fmt::Result {
        write!(fmt, "├")?;

        let dim = self.dimension
            .expect("Dimension not computed");

        let gap = dim.0 - 2;
        write!(fmt, "{}", "─".repeat(gap.into()));
            
        writeln!(fmt, "┤")
    }

    fn format_highlight_item(&self, fmt: &mut String, content: [&str; N]) -> fmt::Result {
        write!(fmt, "│{}",
            color::Bg(color::Blue)
        )?;

        let gaps = self.computed_gaps
            .as_ref()
            .expect("Computed gaps is not initialized");

        for (&g, &s) in gaps.iter().zip(content.iter()){
            write!(fmt, "{:<gap_size$}",
                s,
                gap_size = g.into(),
            )?;
        }

        

        writeln!(fmt, "{}│",
            color::Bg(color::Reset)
        )
    }


    fn format_item(&self, fmt: &mut String, content: &[String; N]) -> fmt::Result {
        write!(fmt, "│",)?;

        let gaps = self.computed_gaps
            .as_ref()
            .expect("Computed gaps is not initialized");

        for (&g, s) in gaps.iter().zip(content.iter()){
            write!(fmt, "{:<gap_size$}",
                s,
                gap_size = g.into(),
            )?;
        }
            
        writeln!(fmt, "│",)
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

    pub fn write(
        &self, 
        stdout: &mut RawTerminal<Stdout>, 
        contents: &[[String; N]], 
        highlight_index: usize
    ) -> Result<(), Box<dyn Error>> {
        let mut buffer = String::new();


        //print header
        self.format_top(&mut buffer)?;
        self.format_table_title(&mut buffer)?;
        self.format_colum_title_bottom(&mut buffer)?;

        //print content
        for (i,content) in contents.into_iter().enumerate(){
            if i == highlight_index {
                self.format_item(&mut buffer, &content)?;
            } else {
                self.format_item(&mut buffer, &content)?;
            }
        }
        
        // print bottom
        self.format_bottom(&mut buffer)?;


        write!(stdout, "{}", buffer)?;
        stdout.flush()?;

        Ok(())
    }
    
    
}

