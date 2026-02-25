pub struct Args {
    pub help: bool,
    pub is_move: bool,
    pub files: Vec<String>,
}

impl Args {
    pub fn parse(args: Vec<String>) -> Self {
        let mut help = false;
        let mut is_move = false;
        let mut files = Vec::new();
        
        for arg in args.iter().skip(1) {
            if arg.starts_with('-') {
                let flag = arg.trim_start_matches('-').to_lowercase();
                match flag.as_str() {
                    "h" | "help" => help = true,
                    "m" | "move" => is_move = true,
                    _ => {}
                }
            } else {
                files.push(arg.clone());
            }
        }
        
        Args { help, is_move, files }
    }
    
    pub fn get_help(&self) -> String {
        "dwag v0.1.0\n\
             Usage: dwag [options] [path]...\n\
             Options:\n\
             \t-h, --help\t\tShow help\n\
             \t-m, --move\t\tMove files instead of copy\n".to_string()
    }
}
