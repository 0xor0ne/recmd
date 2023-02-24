///! Command module
use std::process::Command;

pub struct Cmd {
    c: String,
}

impl Cmd {
    pub fn new(s: &str) -> Self {
        Cmd { c: String::from(s) }
    }

    pub fn run(&self) -> Result<Vec<u8>, String> {
        match shell_words::split(&self.c) {
            Ok(v) => {
                let mut args = Vec::new();
                if v.len() > 1 {
                    args.extend(&v[1..]);
                }
                match Command::new(&v[0]).args(&args).output() {
                    Ok(output) => {
                        let mut out = output.stdout;

                        if !output.stderr.is_empty() {
                            out.extend(output.stderr);
                        }

                        Ok(out)
                    }
                    Err(e) => Err(format!("Failed to execute the command {} ({})", self.c, e)),
                }
            }
            Err(_) => Err("Splitting error".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cmd;

    #[test]
    fn echo() {
        let c = Cmd::new("echo \"test\"");
        match c.run() {
            Ok(o) => {
                let ostr = String::from_utf8_lossy(&o);
                assert_eq!("test\n", ostr);
            }
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn echo_no_new_line() {
        let c = Cmd::new("echo -n \"test\"");
        match c.run() {
            Ok(o) => {
                let ostr = String::from_utf8_lossy(&o);
                assert_eq!("test", ostr);
            }
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn bash() {
        let c = Cmd::new("bash -c 'VAR=test ; echo $VAR'");
        match c.run() {
            Ok(o) => {
                let ostr = String::from_utf8_lossy(&o);
                assert_eq!("test\n", ostr);
            }
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn bash_multiple_opts() {
        let c = Cmd::new("bash -r -c 'VAR=test ; echo $VAR'");
        match c.run() {
            Ok(o) => {
                let ostr = String::from_utf8_lossy(&o);
                assert_eq!("test\n", ostr);
            }
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn command_does_not_exist() {
        let c = Cmd::new("xxxxyyyytttt");
        match c.run() {
            Ok(_) => {
                assert!(false);
            }
            Err(e) => {
                println!("{}", e);
                assert!(true);
            }
        }
    }
}
