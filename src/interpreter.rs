

pub mod ill {
    use std::fs::File;
    use std::io::Read;
    use std::iter::Peekable;
    use std::iter::Cloned;
    use std::str::Chars;
    use std::error::Error;
    use std::fmt;
    use std::fmt::{Display, Formatter};

    const NEWLINE: char = '\n';
    const STACK_DEF: char = '+';
    const STACK_NAME_END_DEF: char= ';';


    #[derive(Default)]
    struct Stack {
        identifier: String,
        value: usize,
    }


    #[derive(Debug)]
    struct EnhancedFile {
        file: File,
        content: String,
    }

    #[derive(Default, Debug)]
    struct ReadHead {
        column: u32,
        line: u32,
    }

    #[derive(Debug)]
    pub enum IllError {
        StackRefinition(ReadHead, String),
        NoStacksFound(File),
    }

    impl Error for IllError {
        fn description(&self) -> &str {
            match *self {
                IllError::StackRefinition(_, _) => "A stack redefinition was attempted.",
                IllError::NoStacksFound(_) => "No stack definitions found.",
                _ => "Unknown Error.",
            }
        }
    }


    impl Display for IllError {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            fn fmt_rh(rh: &ReadHead) -> String { format!("[{}:{}]", rh.line, rh.column) }
            match *self {
                IllError::StackRefinition(ref rh, ref name) => write!(f, "Err@{} => The stack named \"{}\" already exists!", fmt_rh(rh), name),
                IllError::NoStacksFound(ref e_file) => write!(f, "Cannot find a stack definition for {:?}", e_file),
                _ => write!(f, "Undocumented Error."),
            }
        }
    }

    impl ReadHead {
        fn new() -> ReadHead {
            ReadHead { line: 1, column: 1 }
        }
        fn advance_by(&mut self, line: u32, col: u32) {
            self.column += col;
            self.line += line;
        }
        fn advance(&mut self, ch: char) {
            if ch == NEWLINE {
                self.advance_by(1, 0);
                self.column = 0;
            } else {
                self.advance_by(0, 1);
            }
        }
    }

    #[derive(Default)]
    pub struct Interpreter {
        debug: bool,
        files: Vec<EnhancedFile>,
        stacks: Vec<Stack>,
    }

    impl Interpreter {
        pub fn new(debug: bool, sources: Vec<File>) -> Interpreter {
            Interpreter { debug: debug, files: sources.iter().map(|mut f| {
                let mut content = String::new();
                let sz = f.read_to_string(&mut content).unwrap_or(0);
                if debug {
                    println!("[:] read {} bytes for {:?}", sz, f);
                    println!("content = `{}`", content);
                }
                EnhancedFile { file: f.try_clone().expect(&*format!("[ERROR!]: could not create a copy of: {:?}", f)), content: content }
            }).collect(), .. Default::default() }
        }

        fn find_stack(&self, name: String) -> Option<&Stack> {
            self.stacks.iter().find(|x: &&Stack| x.identifier == name)
        }

        fn does_stack_exist(&self, name: String) -> bool {
            self.find_stack(name).is_some()
        }

        fn create_stacks(&mut self) -> Result<(), IllError> {

            fn read_stack_def(it: &mut Peekable<Chars>) -> (u32, String) {
                let z = it.take_while(|c| *c != STACK_NAME_END_DEF).collect::<String>();
                ((z.len() + 1) /* Compensate for missing ';' */ as u32, z.chars().filter(|c| !c.is_whitespace()).collect::<String>())
            } 

            for e_file in &self.files {
                let mut iter = e_file.content.chars().peekable();
                let mut head: ReadHead = ReadHead::new();
                let mut has_found_stacks: bool = false;
                while let Some(x) = iter.next() {
                    head.advance(x);
                    if !x.is_whitespace() {
                        if x == STACK_DEF {
                            has_found_stacks = true;
                            while iter.peek().is_some() && *iter.peek().unwrap() != NEWLINE {
                                let (trav, stack_name) = read_stack_def(iter.by_ref());
                                if self.does_stack_exist(stack_name.clone()) {
                                    let err_str = stack_name.clone();
                                    return Err(IllError::StackRefinition(head, err_str));
                                }
                                if self.debug {
                                    println!("rh @ [{}:{}] Found stack def: and stack: {}", head.line, head.column, stack_name.clone());
                                }
                                head.advance_by(0, trav);
                                self.stacks.push( Stack { identifier: stack_name, ..Default::default()});
                                continue;
                            }
                        }
                    }
                }
                if !has_found_stacks {
                    let file = e_file.file.try_clone().expect("FATAL: Failed to copy file for err formatting.");
                    return Err(IllError::NoStacksFound(file));
                }
            }
            Ok(())
        }

        pub fn begin_parsing(&mut self) -> Result<(), IllError> {
            let res = self.create_stacks();
            if res.is_err() { 
                return res; 
            }
            Ok(())
        }
    }
}