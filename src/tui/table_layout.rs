use crate::tui::{OriginPoint, TerminalSize};

enum GapMode{
    Fixed(u16),
    Float(f32)
}



struct TableLayout{
    width_perc: f32,
    height_perc: f32,

    column_c: u8,
    column_gaps: Option<Vec<GapMode>>,
    title_column: Option<Vec<&'static str>>
}




struct TableDisplay<'a>{
    term_size: &'a TerminalSize,
    origin: OriginPoint,

    dimension: (u16, u16),

    width_perc: f32,
    height_perc: f32,

    column_c: u8,
    column_gaps: Vec<GapMode>,
    title_column: Vec<&'static str>
}


impl TableLayout{

    fn new(width_perc: f32, height_perc: f32, column_c: u8) -> Self {
        Self { 
            width_perc, 
            height_perc, 
            column_c, 
            column_gaps: None, 
            title_column: None 
        }
    }

    #[inline]
    fn default_gaps(col: u8) -> f32 {
        100.0 / col as f32
    }

    #[inline]
    fn default_title() -> &'static str {
        "Unamed"
    } 


    fn set_column_gaps(&mut self, gaps: Vec<GapMode>){
        if gaps.len() != self.column_c.into() {
            panic!("Provided column title does not have the same ammount as the column");
        }

        self.column_gaps = Some(gaps)
    }

    fn set_column_title(&mut self){

    }



    fn build<'a>(self, origin: OriginPoint, term_size: &'a TerminalSize) -> TableDisplay<'a> {
        todo!()
    }



    

}