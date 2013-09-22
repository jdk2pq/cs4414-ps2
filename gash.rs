use std::{io, run};


fn change_dir() {
   // Do stuff
}

fn history() {
   // make history!
}

fn main() {
    static CMD_PROMPT: &'static str = "gash > ";
    let mut hist: ~[~str] = ~[]; 
    
    loop {
        print(CMD_PROMPT);
        let line = io::stdin().read_line();
        hist.push(line.clone());
        debug!(fmt!("line: %?", line));
        let mut argv: ~[~str] = line.split_iter(' ').filter(|&x| x != "")
        .transform(|x| x.to_owned()).collect();
        debug!(fmt!("argv %?", argv));
        
        if argv.len() > 0 {
            let program = argv.remove(0);
            match program {
                ~"exit"     => {return; }
<<<<<<< HEAD
                ~"hist"     => {
                    println(hist.len().to_str());
                    let mut x = 0;
                    while x < hist.len()
                    {
                        println(hist[x]);
                        x++;
                    }   
                }
=======
                ~"cd"       => {change_dir(); }
                ~"history"  => {history(); }
>>>>>>> 68e6da01e6d97002b1efe37ec020c953d5a7c3bc
                _           => {run::process_status(program, argv);}
            }
        }
    }
}
