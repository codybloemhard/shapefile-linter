use std::collections::HashMap;

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum Issue{
    Message(String),
    TWO_PLUS_Z_IN_HEIGHTLINE,
}

pub struct Logger{
    issues: HashMap<Issue,usize>,
}

impl Logger{
    pub fn new() -> Self{
        Logger{
            issues: HashMap::new(),
        }
    }

    pub fn log(&mut self, issue: Issue){
        match issue{
            Issue::Message(string) => { println!("{}", string); },
            x => {
                let res = self.issues.get(&x);
                match res{
                    Some(n) => { self.issues.insert(x, n + 1); },
                    None => { self.issues.insert(x, 1); },
                }
            }
        }
    }

    pub fn report(&self){
        for (issue, count) in self.issues.clone(){
            match issue{
                Issue::Message(_) => {},
                Issue::TWO_PLUS_Z_IN_HEIGHTLINE =>
                    println!("({} times) Heightline consists of multiple Z values!", count),
            }
        }
    }
}
