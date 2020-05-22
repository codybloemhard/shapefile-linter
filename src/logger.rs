use std::collections::HashMap;
// Issues you can give the logger.
#[derive(Clone,PartialEq,Eq,Hash)]
pub enum Issue{
    Message(String),
    TwoPlusZInHeightline,
    UnsupportedShape,
    EmptyShape,
    MultiChunkShape,
    NonOriginBoundingbox,
    EmptyStyleId,
    MissingStyleId,
    PolyNotEnoughVertices,
    OutOfIndicesBound,
    NoEarsLeft,
    InnerNotInside,
}

#[derive(Default)]
pub struct Logger{
    issues: HashMap<Issue,usize>,
    pub debug_panic: bool,
}

impl Logger{
    // Give an issue, A message will be printed and others will be counted.
    pub fn log(&mut self, issue: Issue){
        match issue{
            Issue::Message(string) => { println!("{}", string); },
            x => {
                let res = if let Some(n) = self.issues.get(&x){ *n + 1 }
                else { 1 };
                self.issues.insert(x, res);
            }
        }
    }
    // Print out all the Issues with how often they occured.
    pub fn report(&self){
        for (issue, count) in self.issues.clone(){
            match issue{
                Issue::Message(_) => {},
                Issue::TwoPlusZInHeightline =>
                    println!("({} times) Heightline consists of multiple Z values!", count),
                Issue::UnsupportedShape =>
                    println!("({} times) Currently Unsupported Shape!", count),
                Issue::EmptyShape =>
                    println!("({} times) Empty shape!", count),
                Issue::MultiChunkShape =>
                    println!("({} times) Multi chunk shape!", count),
                Issue::NonOriginBoundingbox =>
                    println!("({} times) Mother chunk left top bounding box is not at origin!", count),
                Issue::EmptyStyleId =>
                    println!("({} times) Empty Style ID!", count),
                Issue::MissingStyleId =>
                    println!("({} times) Missing Style ID!", count),
                Issue::PolyNotEnoughVertices =>
                    println!("({} times) Polygon has less than 3 vertices!", count),
                Issue::OutOfIndicesBound =>
                    println!("({} times) Polygon has more indices than fit in u16!", count),
                Issue::NoEarsLeft =>
                    println!("({} times) Triangulation: no ears left!", count),
                Issue::InnerNotInside =>
                    println!("({} times) Inner polygon not inside any outer polygon!", count),
            }
        }
    }
}
