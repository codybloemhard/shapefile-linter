use std::collections::HashMap;

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum Issue{
    Message(String),
    TwoPlusZInHeightline,
    UnsupportedShapeForHeightmap,
    EmptyShape,
}

#[derive(Default)]
pub struct Logger{
    issues: HashMap<Issue,usize>,
}

impl Logger{
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

    pub fn report(&self){
        for (issue, count) in self.issues.clone(){
            match issue{
                Issue::Message(_) => {},
                Issue::TwoPlusZInHeightline =>
                    println!("({} times) Heightline consists of multiple Z values!", count),
                Issue::UnsupportedShapeForHeightmap =>
                    println!("({} times) Unsupported shape for heightmap!", count),
                Issue::EmptyShape =>
                    println!("({} times) Empty shape!", count),
            }
        }
    }
}
