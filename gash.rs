use std::{io, run};


fn change_dir() {
   // Do stuff
}

fn history() {
   // make history!
}

fn main() {
    static CMD_PROMPT: &'static str = "gash > ";
    
    loop {
        print(CMD_PROMPT);
        let line = io::stdin().read_line();
        debug!(fmt!("line: %?", line));
        let mut argv: ~[~str] = line.split_iter(' ').filter(|&x| x != "")
                                 .transform(|x| x.to_owned()).collect();
        debug!(fmt!("argv %?", argv));
        
        if argv.len() > 0 {
            let program = argv.remove(0);
            match program {
                ~"exit"     => {return; }
                ~"cd"       => {change_dir(); }
                ~"history"  => {history(); }
                _           => {run::process_status(program, argv);}
            }
        }
    }
}
