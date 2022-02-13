use std::{
    ffi::OsStr,
    fs::{self, DirEntry, File},
    io::{BufRead, BufReader, Result},
    os::unix::prelude::OsStrExt,
    process::{Command, Output},
};

const PACMAN_LOG: &str = "/var/log/pacman.log";
const PACMAN_LIB: &str = "/var/cache/pacman/pkg";
const PACMAN_LOG_INSTALLED: &str = " installed ";
const PACMAN_LOG_UPGRADE: &str = " upgraded ";

#[derive(Debug)]
pub struct Filter<'a> {
    date: &'a str,
    key_word: Option<&'a str>,
}
impl Filter<'_> {
    pub fn new(date: &str) -> Filter {
        Filter {
            date,
            key_word: None,
        }
    }

    pub fn new2<'a>(date: &'a str, key: &'a str) -> Filter<'a> {
        Filter {
            date,
            key_word: Some(key),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum PkgType {
    Upgraded,
    Installed,
}

#[derive(Debug)]
pub struct Pkg {
    name: String,
    version: String,
    pub p_type: PkgType,
}

impl Pkg {
    fn get_pkg(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

pub fn execute(filter: &Filter) -> Vec<Pkg> {
    let file = File::open(PACMAN_LOG).expect("open pacman_log failed");
    let reader = BufReader::new(file);

    let result: Vec<Pkg> = reader
        .lines()
        .filter(|line| line.is_ok())
        .map(|line| line.expect("file string error"))
        .filter(|line| {
            let start = format!("[{}", filter.date);
            line.starts_with(&start)
        })
        .filter(|line| {
            if !line.contains(PACMAN_LOG_INSTALLED) && !line.contains(PACMAN_LOG_UPGRADE) {
                return false;
            }

            if let Some(key) = filter.key_word {
                line.contains(key)
            } else {
                true
            }
        })
        .map(|line| convert(line.as_str()))
        .collect();
    result
}

pub fn do_uninstalled(installed: Vec<&Pkg>) {
    if !installed.is_empty() {
        for pkg in installed {
            match uninstall(pkg) {
                Ok(_) => {}
                Err(e) => {
                    println!("{} uninstall error, message:{}", pkg.get_pkg(), e);
                }
            };
        }
    }
}

pub fn do_downgrade(upgraded: Vec<&Pkg>) {
    if !upgraded.is_empty() {
        let files = fs::read_dir(PACMAN_LIB).expect("read pacman lib failed");
        let ds: Vec<DirEntry> = files
            .filter_map(Result::ok)
            .filter(|d| d.path().extension() == Some(OsStr::from_bytes(b"pkg.tar.zst")))
            .collect();

        for pkg in upgraded {
            let p_name = &pkg.get_pkg();
            let f = ds
                .iter()
                .find(|d| (*d).file_name().to_str().unwrap().starts_with(p_name));
            if let Some(c) = f {
                let d_result = downgrade(c.path().to_str().expect("execute failed"));

                match d_result {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{} downgrade error, message:{}", p_name, e)
                    }
                }
            } else {
                println!("pkg:{} not found in local", p_name);
            }
        }
    }
}

fn downgrade(path: &str) -> Result<Output> {
    Command::new("pacman").arg("-Udd").arg(path).output()
}

fn uninstall(pkg: &Pkg) -> Result<Output> {
    Command::new("pacman")
        .arg("-R")
        .arg(pkg.get_pkg().as_str())
        .output()
}
// [2022-01-23T10:31:28+0800] [ALPM] installed opencc (1.1.2-3)
// [2022-02-12T12:02:12+0800] [ALPM] upgraded firefox (96.0.3-1 -> 97.0-0.1)
fn convert(line: &str) -> Pkg {
    if line.contains(PACMAN_LOG_INSTALLED) {
        let split: Vec<&str> = line.split(PACMAN_LOG_INSTALLED).collect();
        let p_str = *split.last().unwrap();
        let ps: Vec<&str> = p_str.split_whitespace().collect();
        Pkg {
            name: String::from(ps[0]),
            version: String::from(&ps[1][1..ps[1].len() - 1]),
            p_type: PkgType::Installed,
        }
    } else if line.contains(PACMAN_LOG_UPGRADE) {
        let split: Vec<&str> = line.split(PACMAN_LOG_UPGRADE).collect();
        let p_str = *split.last().unwrap();
        let ps: Vec<&str> = p_str.split_whitespace().collect();
        // println!("{:?}", ps);

        Pkg {
            name: String::from(ps[0]),
            version: String::from(&ps[1][1..ps[1].len()]),
            p_type: PkgType::Upgraded,
        }
    } else {
        panic!("")
    }
}

#[cfg(test)]
mod test {
    use super::convert;

    // pub fn auto_downgrade(filter: &Filter) -> Result<()> {
    //     println!("{},{}", PACMAN_LOG, PACMAN_LIB);
    //     let file = File::open(PACMAN_LOG)?;
    //     let reader = BufReader::new(file);

    //     let result: Vec<Pkg> = reader
    //         .lines()
    //         .filter(|line| line.is_ok())
    //         .map(|line| line.expect("file string error"))
    //         .filter(|line| {
    //             let start = format!("[{}", filter.date);
    //             line.starts_with(&start)
    //         })
    //         .filter(|line| {
    //             if !line.contains(PACMAN_LOG_INSTALLED) && !line.contains(PACMAN_LOG_UPGRADE) {
    //                 return false;
    //             }

    //             if let Some(key) = filter.key_word {
    //                 line.contains(key)
    //             } else {
    //                 true
    //             }
    //         })
    //         .map(|line| convert(line.as_str()))
    //         .collect();

    //     let installed: Vec<&Pkg> = result
    //         .iter()
    //         .filter(|x| x.p_type.eq(&PkgType::Installed))
    //         .collect();

    //     let upgraded: Vec<&Pkg> = result
    //         .iter()
    //         .filter(|x| x.p_type.eq(&PkgType::Upgraded))
    //         .collect();

    //     if !upgraded.is_empty() {
    //         let files = fs::read_dir(PACMAN_LIB)?;
    //         let ds: Vec<DirEntry> = files
    //             .filter_map(Result::ok)
    //             .filter(|d| d.path().extension() == Some(OsStr::from_bytes(b"pkg.tar.zst")))
    //             .collect();

    //         for pkg in upgraded {
    //             let p_name = &pkg.get_pkg();
    //             let f = ds
    //                 .iter()
    //                 .find(|d| (*d).file_name().to_str().unwrap().starts_with(p_name));
    //             if let Some(c) = f {
    //                 let d_result = downgrade(c.path().to_str().expect("execute failed"));

    //                 match d_result {
    //                     Ok(_) => {}
    //                     Err(e) => {
    //                         println!("{} downgrade error, message:{}", p_name, e)
    //                     }
    //                 }
    //             } else {
    //                 println!("pkg:{} not found in local", p_name);
    //             }
    //         }
    //     }

    //     if !installed.is_empty() {
    //         for pkg in installed {
    //             match uninstall(pkg) {
    //                 Ok(_) => {}
    //                 Err(e) => {
    //                     println!("{} uninstall error, message:{}", pkg.get_pkg(), e);
    //                 }
    //             };
    //         }
    //     }

    //     Ok(())
    // }

    #[test]
    fn test_convert_installed() {
        let installed = "[2022-01-23T10:31:28+0800] [ALPM] installed opencc (1.1.2-3)";
        let p = convert(installed);
        assert_eq!(p.name, "opencc");
        assert_eq!(p.version, "1.1.2-3");
        assert_eq!(p.p_type, super::PkgType::Installed);
        assert_eq!(p.get_pkg(), "opencc-1.1.2-3");
    }

    #[test]
    fn test_convert_upgraded() {
        let upgraded = "[2022-02-12T12:02:12+0800] [ALPM] upgraded firefox (96.0.3-1 -> 97.0-0.1)";
        let p1 = convert(upgraded);
        assert_eq!(p1.name, "firefox");
        assert_eq!(p1.version, "96.0.3-1");
        assert_eq!(p1.p_type, super::PkgType::Upgraded);
    }
}
