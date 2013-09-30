use std::{io, str, run, os, libc};
use std::run::{Process, ProcessOptions};

struct p_params  { // stores information to be passed into a process
    program: ~str,
    args: ~[~str],
    in_fd:    Option<i32>,
    out_fd:   Option<i32>,
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

fn readclose(fd: libc::c_int) -> ~str {
    unsafe {
        let file = os::fdopen(fd);
        let reader = io::FILE_reader(file, false);
        let buf = reader.read_whole_stream();
        os::fclose(file);
        str::from_bytes(buf)
    }
}

fn writeclose(fd: libc::c_int, s: &str) {
    let writer = io::fd_writer(fd, false);
    writer.write_str(s);
    os::close(fd);
}


fn parse_and_run(program: &str, args: &[~str]) { // will eventually need to return an int
    let args = args.to_owned();
    let mut i = 0; 
    let mut list_pos = 0;
    let mut pipes: ~[os::Pipe] = ~[os::pipe(), ];
    let mut first_p = pipes[0];
    let mut last_p = os::pipe();
    let mut progs_to_run: ~[p_params] = ~[p_params { program: program.to_owned(), args: ~[], in_fd: Some(first_p.in), out_fd: Some(last_p.out)}];
    while list_pos < args.len() {
        let arg: ~str = args[list_pos].to_owned();
        let mut cur_prog = copy progs_to_run[i];
        match arg {
            ~"<"    => {
                let filename: &str = args[list_pos + 1];
                let FILE_fd: i32 = read_file_fd(filename);
                cur_prog.in_fd =  Some(FILE_fd);
                progs_to_run[i] = cur_prog; 
                list_pos += 1;  
            },
            ~">"    => {
                let filename: &str = args[list_pos + 1];
                let FILE_fd: i32 = write_file_fd(filename);
                cur_prog.out_fd =  Some(FILE_fd);
                progs_to_run[i] = cur_prog; 
                list_pos += 1;  
            },
            ~"|"    => {
                let pipe =  os::pipe();
                pipes.push(pipe);
                println(fmt!("%?", pipe));
                cur_prog.out_fd = Some(pipe.out);
                progs_to_run[i] = cur_prog;
                let next_pstruct = p_params { program: ~"", args: ~[], in_fd: Some(pipe.in), out_fd: Some(last_p.out)};
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
    println(fmt!("%?", progs_to_run));
    while j <  progs_to_run.len()  {
        let cur = copy progs_to_run[j];
        println("before");
            let p =  &'static Process::new(cur.program, cur.args, ProcessOptions { 
                env: None,
                dir: None,
                in_fd:  cur.in_fd,
                out_fd: cur.out_fd,
                err_fd: Some(error_p.out)
            }); 
        println("after");
        j += 1;
    };

    os::close(last_p.out);
    println(fmt!("closing pipes %?", pipes));
    for pipes.iter().advance() |pipe| {
        println(fmt!("closing pipes %?", pipe));
        os::close(pipe.out);
        os::close(pipe.in);
    };
    os::close(error_p.out);
    os::close(error_p.in);
    let actual = readclose(last_p.in);
    println(actual);

}

fn main() {
    // `static indicates that variable will have the static lifetime, meaing
    // it gets to live for the whole life of the program
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
                    parse_and_run("sudo", args);
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
        hist.push(line.clone());
        if (argv.len() >= 1) {
            let end = argv.len() - 1;
            lastarg = argv.remove(end);
        }
        cwd = os::getcwd().to_str();
    }
}
