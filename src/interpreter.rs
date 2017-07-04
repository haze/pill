
const NEWLINE: char = '\n';
const STACK_DEF: char = '+';
const STACK_NAME_END_DEF: char= ';';

pub mod ill {
    use std::fs::File;
    use std::io::Read;
    struct Stack {
        identifier: String,
        value: usize,
    }


    struct EnhancedFile {
        file: File,
        content: String,
    }

    #[derive(Default)]
    struct ReadHead {
        column: u32,
        row: u32,
    }

    impl ReadHead {
        fn advance(&self, ch: char) {
            if ch == NEWLINE {
                self.row += 1;
                self.column = 0;
            } else {
                self.column += 1;
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

        fn create_stacks(&self) {
            for e_file in &self.files {
                let mut iter = e_file.content.chars().peekable();
                let mut head: ReadHead = Default::default();
                let mut has_found_stacks: bool = false;
                let mut stack_name_buf: String = String::new();
                while let Some(x) = iter.next() {
                    if x == STACK_DEF {
                        has_found_stacks = true;
                        let stack_name = iter.take_while(|x| x != STACK_NAME_END_DEF).collect();
                    }
                    head.advance(x);
                }
                if !has_found_stacks {
                    panic!("[ERROR!]: found no stacks (or definitions) while parsing file {:?}", f);
                }
            }
        }

        pub fn begin_parsing(&self) {
            self.create_stacks();
        }
    }
}