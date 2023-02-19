use shell_words;
///! Command module
use std::process::Command;

pub struct Cmd {
    c: String,
}

impl Cmd {
    pub fn new(s: &str) -> Self {
        Cmd { c: String::from(s) }
    }

    pub fn run(&self) -> Result<String, String> {
        match shell_words::split(&self.c) {
            Ok(v) => {
                let mut args = Vec::new();
                if v.len() > 1 {
                    args.extend(&v[1..]);
                }
                match Command::new(&v[0]).args(&args).output() {
                    Ok(output) => {
                        let mut out = String::from_utf8_lossy(&output.stdout);
                        let err = String::from_utf8_lossy(&output.stderr);

                        if err.len() > 0 {
                            out.to_mut().push('\n');
                            out.to_mut().push_str(&err);
                        }

                        Ok(out.to_string())
                    }
                    Err(e) => Err(format!(
                        "Failed to execute the command {} ({})",
                        self.c,
                        e.to_string()
                    )),
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
                assert_eq!("test\n", o);
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
                assert_eq!("test", o);
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
                assert_eq!("test\n", o);
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
            Ok(o) => {
                assert!(false);
            }
            Err(e) => {
                println!("{}", e);
                assert!(true);
            }
        }
    }
}
