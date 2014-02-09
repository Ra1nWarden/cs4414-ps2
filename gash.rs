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
use std::io::signal::{Listener, Interrupt};
use std::libc;
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
                _       =>  { self.run_pipes(cmd_line); }
            }
        }
    }
    
    fn run_pipes(&mut self, cmd_line: &str) {
       let progs: ~[~str] = 
       	       	      cmd_line.split('|').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
       let mut channels: ~[std::os::Pipe] = ~[];
       for _ in range(0, progs.len()) {
       	   channels.push(std::os::pipe());
       }
       for i in range(0, progs.len()) {
       	   let mut in_chan = libc::STDIN_FILENO;
	   let mut out_chan = libc::STDOUT_FILENO;
	   if i == 0 {
	      out_chan = channels[i].out;
	   }
       	   if i > 0 {
	      in_chan = channels[i-1].input;
	      out_chan = channels[i].out;
	   }
	   if i == progs.len() - 1 {
	      out_chan = libc::STDOUT_FILENO;
	   }
	   self.run_cmdline(progs[i].trim().clone(), in_chan, out_chan);
       }
    }
    
    fn run_cmdline(&mut self, cmd_line: &str, in_chan: libc::c_int, out_chan: libc::c_int) {	 
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

	   // Handle IO redirection
	   let mut input: libc::c_int = in_chan;
	   let mut output: libc::c_int = out_chan;
	   let argv_length = argv.len();
	   for i in range(0, argv_length) {
	       if i >= argv.len() {
	       	  break;
	       }
	       if argv[i] == ~"<" {
	       	  argv.remove(i);
	       	  if i >= argv.len() {
		     println!("No file specified for input.")
		     return;
		  }
		  else {
		       unsafe{
			let in_file_name = argv.remove(i);
		       	let mode = "r".to_c_str().unwrap();
			let file_path = Path::new(in_file_name.clone());
			if file_path.exists() {	
			   let name_in_str = in_file_name.clone().to_c_str().unwrap();
			   let in_file = libc::fopen(name_in_str, mode);
		       	   input = libc::fileno(in_file);
			}
			else {
			   println!("Input file does not exists!");
			   return;
			}
		       }
		  }
	       }
	       if i >= argv.len() {
	       	  break;
	       }
	       if argv[i] == ~">" {
	       	  argv.remove(i);
		  if i >= argv.len() {
		     println!("No file specified for output.")
		     return;
		  }
		  else {
		       unsafe {
		       	 let out_file_name = argv.remove(i).to_c_str().unwrap();
		       	 let mode = "w".to_c_str().unwrap();
		       	 let out_file = libc::fopen(out_file_name, mode);
		       	 output = libc::fileno(out_file);
		       }
		     if output == -1 {
		     	println!("Invalid file for output.");
			return;
		     }
		  }
	       }
	   }
	   
	   if background {
	      let prog = mod_prog.clone();
	      let arguments = argv.clone();
	      let fin_chan = input.clone();
	      let fout_chan = output.clone();
	      let err_chan = libc::STDERR_FILENO;
	      if self.cmd_exists(prog) {
	      	 do spawn {
		    let proc_run_opt = run::Process::new(prog, arguments, run::ProcessOptions {
		    		  			  	     env: None,
								     dir: None,
								     in_fd: Some(fin_chan),
								     out_fd: Some(fout_chan),
								     err_fd: Some(err_chan)
		    		  			  	     });
		    let mut proc_run = proc_run_opt.unwrap();
		    let mut listener = Listener::new();
		    listener.register(Interrupt);
		    proc_run.finish();
		    if fin_chan != 0 {
		       std::os::close(fin_chan);
		    }
		    if fout_chan != 1 {
		       std::os::close(fout_chan);
		    }
		 }
	      }
	      else {
	      	 println!("{:s}: command not found", prog);
	      }
	   }
	   else {
	   	self.run_cmd(program, argv, input, output);
	   }
        }
    }
    
    fn run_cmd(&mut self, program: &str, argv: &[~str], input_chan: libc::c_int, output_chan: libc::c_int) {
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
            	  let proc_run_opt = run::Process::new(program, argv, run::ProcessOptions {
		    		  			  	     env: None,
								     dir: None,
								     in_fd: Some(input_chan),
								     out_fd: Some(output_chan),
								     err_fd: Some(libc::STDERR_FILENO)
		    		  			  	     });
		  let mut proc_run = proc_run_opt.unwrap();
		  let mut listener = Listener::new();
		  listener.register(Interrupt);
		  proc_run.finish();
		  if input_chan != 0 {
		     std::os::close(input_chan);
		  }
		  if output_chan != 1 {
		     std::os::close(output_chan);
		  }
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
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line, libc::STDIN_FILENO, libc::STDOUT_FILENO),
        None           => Shell::new("gash > ").run()
    }
}
