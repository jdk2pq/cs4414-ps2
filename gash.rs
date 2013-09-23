use std::{io, run, os};

fn main() {
    static CMD_PROMPT: &'static str = "gash > ";
    let mut hist: ~[~str] = ~[]; 
    
    loop {
        print(CMD_PROMPT);
        let line = io::stdin().read_line();
        hist.push(line.clone());
        debug!(fmt!("line: %?", line));
        let mut argv: ~[~str] = line.split_iter(' ').filter(|&x| x != "").transform(|x| x.to_owned()).collect();
        debug!(fmt!("argv %?", argv));
        
        if argv.len() > 0 {
            let program = argv.remove(0);
            match program {
                ~"exit"     => {return; }
                ~"cd"       => {
                    if argv.len() == 0 {
                        let homedir = match os::homedir() {
                            Some(m) => m,
                            None => GenericPath::from_str("~")
                        };
                        os::change_dir(&homedir);    
                    } else {
                        let dir: &Path = &GenericPath::from_str(argv.remove(0));
                        if !os::change_dir(dir) { 
                            println("Error: No such file or directory");
                        }
                    }
                }
                ~"history"  => {
                    let mut x = 0;
                    while x < hist.len()
                    {
                        println(hist[x]);
                        x += 1;
                    }   
                }
                _           => {
                    if argv.len() != 0 {
                        let mut background = ~"";
                        if argv.last() == &~"&" {
                            background = argv.pop();
                        } else {
                            background = ~"";
                        }
                        let args = argv;
                        match background {
                            ~"&" => {
                                do std::task::spawn_sched(std::task::SingleThreaded) { 
                                    run::Process::new(program, args, run::ProcessOptions::new());
                                }
                            }
                            _ => {run::process_status(program, args);}
                        }
                    } else {
                        run::process_status(program, argv);
                    }

                }
            }
        }
    }
}
