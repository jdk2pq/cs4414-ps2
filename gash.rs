use std::{io, str, run, os, libc};

struct p_params  { // stores information to be passed into a process
    program: ~str,
    args: ~[~str],
    in_fd:    i32,
    out_fd:   i32,
}

fn read_file_fd(filename: &str) -> i32 {
    let test: &Path = &GenericPath::from_str(filename);
    if !os::path_exists(test) { 
        fail!("no such file"); 
    }
    unsafe {
        do os::as_c_charp(filename) |filename| {
            do os::as_c_charp ("r") |mode| { 
                let FILE = libc::fopen(filename, mode); //A ptr to a FILE struct
                libc::fileno(FILE)
            } 
        }
    }
}

fn write_file_fd(filename: &str) -> i32 {
    unsafe {
        do os::as_c_charp(filename) |filename| {
            do os::as_c_charp ("w") |mode| { 
                let FILE = libc::fopen(filename, mode); //A ptr to a FILE struct
                libc::fileno(FILE)
            } 
        }
    }
}

// reads a given file descriptor and closes
fn read_close(fd: libc::c_int) -> ~str {
    unsafe {
        let file = os::fdopen(fd);
        let reader = io::FILE_reader(file, false);
        let buf = reader.read_whole_stream();
        os::fclose(file);
        str::from_bytes(buf)
    }
}

fn process_backticks(orig_args: &[~str]) -> ~[~str] { // launches subprocesses
    let args = orig_args.to_owned();
    let mut ticks = false;
    let mut return_args: ~[~str] = ~[];
    let mut sub_proc = ~[];
    let mut i = 0;
    // iteratively process argument for backticks
    while i < args.len() {
       let mut arg: ~str = args[i].to_owned();
       if arg.starts_with("`") {
          ticks = true;
          arg = arg.slice_from(1).to_owned();
       }     
       if ticks {
          if arg.ends_with("`") {
            arg = arg.slice_to(arg.len()-1).to_owned();
            sub_proc.push(arg);
            // Evaluate command specified by tick
            let program = copy sub_proc[0];              
            let pass_in = copy sub_proc.slice(1, sub_proc.len()).to_owned();
            // this is where a new process, which lives in backticks is spawned
            let mut retd_args_str = parse_and_run(program, pass_in, true);
            // A hack to format string correctly for processing
            retd_args_str = retd_args_str.replace("\n", " ");
            // Take result and add it to arg str
            let retd_args: ~[~str] = retd_args_str.split_iter(' ').filter(|&x| x != "").transform(|x| x.to_owned()).collect();
            for retd_args.iter().advance |r_arg| {
                return_args.push(r_arg.to_owned());
            }
            // reset tick and sub_proc to process multiple ticks
            ticks = false;
            sub_proc = ~[];
          } else {
              sub_proc.push(arg);
          }
       } else {
         return_args.push(arg)
       }
       i += 1;
    } 
    return return_args;
} // returns the args with backticks evaluated and filled in  


fn parse_and_run(program: &str, args: &[~str], return_stdout: bool) -> ~str { // return_stdout allows us to play with pipe or process output
    let mut args = args.to_owned();
    // Extra Feature! backticks!
    args = process_backticks(args);
    // REGULAR PROCESSING
    let mut i = 0; 
    // This list keeps track of all the pipes we are going to use, (we must close them later)
    let mut pipes: ~[os::Pipe] = ~[];
    let  first_p = os::pipe();
    let  last_p = os::pipe();
    // holds our process structs                     The first process here reads from stdin: hence V
    let mut progs_to_run: ~[p_params] = ~[p_params { program: program.to_owned(), args: ~[], in_fd: 0, out_fd: last_p.out}];
    // poorly named, for index into p_param array
    let mut list_pos = 0;
    // iteratively consider each argument
    while list_pos < args.len() {
        let arg: ~str = args[list_pos].to_owned();
        let mut cur_prog = copy progs_to_run[i];
        match arg {
            ~"<"    => {
                let filename: &str = args[list_pos + 1];
                let FILE_fd: i32 = read_file_fd(filename);
                cur_prog.in_fd =  FILE_fd;
                progs_to_run[i] = cur_prog; 
                list_pos += 1;  
            },
            ~">"    => {
                let filename: &str = args[list_pos + 1];
                let FILE_fd: i32 = write_file_fd(filename);
                cur_prog.out_fd =  FILE_fd;
                progs_to_run[i] = cur_prog; 
                list_pos += 1;  
            },
            ~"|"    => {
                // this pipe will be recycled
                let pipe =  os::pipe();
                pipes.push(pipe);
                cur_prog.out_fd = pipe.out;
                progs_to_run[i] = cur_prog;
                let next_pstruct = p_params { program: ~"", args: ~[], in_fd: pipe.in, out_fd: last_p.out};
                progs_to_run.push(next_pstruct);
                i += 1;
            },
            _       => {
                if cur_prog.program != ~"" {
                    cur_prog.args.push(arg);
                } else {
                    cur_prog.program = arg;
                }
                progs_to_run[i] = cur_prog;
            },
        }
        list_pos = list_pos + 1;
    }

    let mut j = 0;
    let error_p = os::pipe();
    // iterate through p_param struct to spawn processes
    while j <  progs_to_run.len()  {
        let cur = copy progs_to_run[j];
        // This spawns our subprocesses
        run::spawn_process_os(cur.program, 
                              cur.args,
                              None, None, 
                              cur.in_fd,
                              cur.out_fd,
                              error_p.out);
        j += 1;
    };

    // we must close all open file descriptors of parent process
    os::close(last_p.out);
    os::close(first_p.in);
    for pipes.iter().advance() |pipe| {
        os::close(pipe.out);
        os::close(pipe.in);
    };
    os::close(error_p.out);
    // branch on return type, usually we just print to screen
    if return_stdout {
        println(read_close(error_p.in));
        return read_close(last_p.in);   
    } else {
        println(read_close(error_p.in));
        println(read_close(last_p.in));
        return ~"";
    }   
    
}

fn main() {
    let mut cwd: ~str = os::getcwd().to_str();
    let mut hist: ~[~str] = ~[]; 
    let mut lastarg: ~str = ~"";
    
    loop {
        let CMD_PROMPT: ~str = cwd + " gash > ";
        print(CMD_PROMPT);
        let mut line = io::stdin().read_line();
        line = line.replace("!$", lastarg);
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
                ~"!!"  => {
                    let end = hist.len() -1;
                    let lastprog = hist.remove(end);
                    let args: ~[~str] = lastprog.split_iter(' ').filter(|&x| x != "").transform(|x| x.to_owned()).collect();
                    parse_and_run("sudo", args, false);
                }
                _           => {
                    let dir: &Path = &GenericPath::from_str(line);
                    if !os::change_dir(dir) { 
                        if argv.len() != 0 {
                            let mut background: ~str;
                            if argv.last() == &~"&" {
                                background = argv.pop();
                            } else {
                                background = ~"";
                            }
                            // this line works because it changes the mutability of argv
                            // so that the subproccess can confidently access values in argv
                            let args: ~[~str] = copy argv; 
                            // we can no longer modify argv, hence:
                            // argv[0] = ~"foo"; //fails complitation
                            match background {
                                ~"&" => {
                                    do std::task::spawn_sched(std::task::SingleThreaded) { 
                                        parse_and_run(program, args, false);
                                    }
                                }
                                _ => {parse_and_run(program, args, false);}
                            }
                        } else {
                            parse_and_run(program, argv, false);
                        }
                    }
                }
            }
        }
        hist.push(line.clone());
        if (argv.len() >= 1) {
            let end = argv.len() - 1;
            lastarg = argv.remove(end);
        }
        cwd = os::getcwd().to_str();
    }
}
