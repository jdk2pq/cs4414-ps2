use std::{io, run, os, vec};

struct p_params  { // stores information to be passed into a process
    program: ~str,
    args: ~[~str]
}

fn parse_and_run(program: &str, args: &[~str]) { // will eventually need to return an int
    // deal with args to figure out what needs to be done with fds
    let mut first = p_params { program: program.to_owned(), args: ~[]};
    let mut progs_to_run: ~[p_params] = ~[first];
    let mut i = 0; 
    for args.iter().advance |arg| {
        let mut arg: ~str = arg.to_owned();
        match arg {
            ~"<"    => {},
            ~">"    => {},
            ~"|"    => {
                let next_pstruct = p_params { program: ~"", args: ~[]};
                progs_to_run.push(next_pstruct);
                i += 1;
                // deal with pipe setup
            },
            _       => {
                // must copy out and in for some reason
                // ASK SOMEONE DONT FORGET
                // remove copy compiler throws:
                // error: cannot move out of captured outer variable 
                let mut cur_prog = copy progs_to_run[i];
                if cur_prog.program != ~"" {
                    println(fmt!("%?", arg));
                    cur_prog.args.push(arg);
                } else {
                    cur_prog.program = arg;
                }
                progs_to_run[i] = cur_prog;
            },
        }
    }
    println(fmt!("%?", progs_to_run));

    // spawn process
    run::process_status(progs_to_run[0].program, progs_to_run[0].args);
}


fn main() {
    // `static indicates that variable will have the static lifetime, meaing
    // it gets to live for the whole life of the program
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
                        let mut background: ~str;
                        if argv.last() == &~"&" {
                            background = argv.pop();
                        } else {
                            background = ~"";
                        }
                        // this line works because it changes the mutability of argv
                        // so that the subproccess can confidently access values in argv
                        let args: ~[~str] = argv; 
                        // we can no longer modify argv, hence:
                        // argv[0] = ~"foo"; //fails complitation
                        match background {
                            ~"&" => {
                                do std::task::spawn_sched(std::task::SingleThreaded) { 
                                    parse_and_run(program, args);
                                }
                            }
                            _ => {parse_and_run(program, args);}
                        }
                    } else {
                        parse_and_run(program, argv)
                    }

                }
            }
        }
    }
}
