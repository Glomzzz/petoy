use std::collections::LinkedList;
use std::error::Error;
use std::io::Write;

struct Console{
    lines:Vec<String>
}

pub(crate) struct ConsoleManager {
    consoles:LinkedList<Console>
}

impl ConsoleManager {
    pub fn new() -> Self{
        Self{
            consoles:[Console::new()].into_iter().collect()
        }
    }

    pub fn head(&mut self) -> &mut Console {
        self.consoles.front_mut().unwrap()
    }

    pub fn join(&mut self,todo: impl Fn(&mut Console) -> ()){
        let mut console = Console::new();
        todo(&mut console);
        self.consoles.push_front(console);
        self.flush();
    }

    pub  fn leave(&mut self){
        self.consoles.pop_front();
        self.flush();
    }
    pub fn print(&mut self,line:impl Into<String>){
        self.head().print(line.into());
        self.flush();
    }

    pub fn println(&mut self,line:impl Into<String>){
        self.head().println(line.into());
        self.flush();
    }
    pub fn clear(&mut self){
        self.head().clear();
        print!("\x1B[2J");
    }

    pub fn flush(&mut self){
        print!("\x1B[2J");
        for line in &self.head().lines {
            print!("{}", line);
        }
        std::io::stdout().flush().unwrap()
    }
    pub fn input(&self) -> Result<String,Box<dyn Error>> {
        print!(">> ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        Ok(input.trim_end().into())
    }
}

impl Console {
    fn new() -> Self {
        Self {
            lines: Vec::new()
        }
    }

   pub fn print(&mut self, line: String) {
        self.lines.push(line);
    }

    pub fn println(&mut self, line: String) {
        self.lines.push(line + "\n");
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

}