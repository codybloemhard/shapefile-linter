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
}

#[derive(Default)]
pub struct Logger{
    issues: HashMap<Issue,usize>,
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
            }
        }
    }
}
