//
// gash.rs
//
// Starting code for PS2
// Running on Rust 0.9
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu, David Evans
// Version 0.4
//

extern mod extra;

use std::{io, run, os};
use std::io::buffered::BufferedReader;
use std::io::stdin;
use extra::getopts;

struct Shell {
    cmd_prompt: ~str,
    cmd_history: ~[~str]
}

impl Shell {
    fn new(prompt_str: &str) -> Shell {
        Shell {
            cmd_prompt: prompt_str.to_owned(),
	    cmd_history: ~[]
        }
    }
    
    fn run(&mut self) {
        let mut stdin = BufferedReader::new(stdin());
        
        loop {
            print(self.cmd_prompt);
            io::stdio::flush();
            
            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim().to_owned();
	    self.cmd_history.push(cmd_line.clone());
            let program = cmd_line.splitn(' ', 1).nth(0).expect("no program");
            
	    let mut mod_prog = program.clone();
	    let prog_length = mod_prog.len();
	    if mod_prog.slice_from(prog_length - 1) == "&" {
	       mod_prog = mod_prog.slice_to(prog_length - 1);
	    }	    

            match mod_prog {
                ""      =>  { continue; }
                "exit"  =>  { return; }
                _       =>  { self.run_cmdline(cmd_line); }
            }
        }
    }
    
    fn run_cmdline(&mut self, cmd_line: &str) {	 
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        if argv.len() > 0 {
	   let program: ~str = argv.remove(0);
	   let mut mod_prog = program.clone();
	   let mut background = false;
	   if argv.len() == 0 {
	      let prog_length = mod_prog.len();    
	      if mod_prog.slice_from(prog_length - 1) == "&" {
	      	 mod_prog = mod_prog.slice_to(prog_length - 1).trim().to_owned();
	       	 background = true;
	      }
	   }
	   else {
	      let mut last_arg = argv[argv.len() - 1].clone();
	      if last_arg == ~"&" {
	      	 argv.pop();
		 background = true;
	      }
	      else {
	      	   let arg_length = last_arg.len();
	      	   if last_arg.slice_from(arg_length - 1) == "&" {
	      	      last_arg = last_arg.slice_to(arg_length - 1).to_owned();
		      argv.pop();
		      argv.push(last_arg);
		      background = true;
	      	   } 
	      }  
	   }
	   if background {
	      let prog = mod_prog.clone();
	      let arguments = argv.clone();
	      if self.cmd_exists(prog) {
	      	 do spawn {
		    run::process_status(prog, arguments);
		 }
	      }
	      else {
	      	 println!("{:s}: command not found", prog);
	      }
	   }
	   else {
	   	self.run_cmd(program, argv);
	   }
        }
    }
    
    fn run_cmd(&mut self, program: &str, argv: &[~str]) {
        if self.cmd_exists(program) {
	    if program == "cd" {
	       if argv.len() > 1 {
	       	  println!("Usage: cd <directory>!");
	       }
	       else if argv.len() == 0 {
	       	  let tar_path = std::os::homedir().unwrap();
		  os::change_dir(&tar_path);
	       }
	       else {
	       	  let tar_path = Path::new(argv[0].clone());
		  match os::change_dir(&tar_path) {
		  	true => { return; }
			false => { println!("No such directory!"); }
		  }
	       }
	    }
	    else {
            	 run::process_status(program, argv);
            }
        } 
	else if program == "history" {
	     for i in range(0, self.cmd_history.len()) {
	     	 println!(" {:u} {:s}", i+1, self.cmd_history[i]);
	     }
	}
	else {
            println!("{:s}: command not found", program);
        }
    }
    
    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }
}

fn get_cmdline_from_args() -> Option<~str> {
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    
    let opts = ~[
        getopts::optopt("c")
    ];
    
    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };
    
    if matches.opt_present("c") {
        let cmd_str = match matches.opt_str("c") {
                                                Some(cmd_str) => {cmd_str.to_owned()}, 
                                                None => {~""}
                                              };
        return Some(cmd_str);
    } else {
        return None;
    }
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();
    
    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line),
        None           => Shell::new("gash > ").run()
    }
}
