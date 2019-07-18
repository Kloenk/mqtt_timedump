use clap::{App, Arg, SubCommand};

fn main() {
    let mut app = App::new("timedumper")
        .version(env!("CARGO_PKG_VERSION")) // load version from cargo
        .author("Kloenk <me@kloenk.de>")
        .about("dump time to mqtt topic")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("broker")
                .short("H")
                .long("host")
                .help("set mqtt broker host")
                .value_name("IP"),
        )
        .arg(
            Arg::with_name("user")
                .short("u")
                .long("user")
                .help("set user for mqtt broker")
                .value_name("NAME"),
        )
        .arg(
            Arg::with_name("pass")
                .short("P")
                .long("password")
                .help("set mqtt broker password")
                .value_name("PASSWORD"),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("set a config file")
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("set the port the mqtt broker")
                .value_name("PORT"),
        )
        .arg(
            Arg::with_name("updates")
                .short("t")
                .long("updated")
                .help("set how many seconds between updates")
                .value_name("SECONDS"),
        )
        .arg(
            Arg::with_name("topic")
                .short("T")
                .long("topic")
                .help("mqtt topic")
                .value_name("NAME"),
        )
        .subcommand(
            SubCommand::with_name("completion")
                .about("create completions")
                .version("0.1.0")
                .author("Kloenk <me@kloenk.de>")
                .arg(
                    Arg::with_name("shell")
                        .help("set the shell to create for. Tries to identify with env variable")
                        .index(1)
                        .required(false)
                        .value_name("SHELL")
                        .possible_value("fish")
                        .possible_value("bash")
                        .possible_value("zsh")
                        .possible_value("powershell")
                        .possible_value("elvish"),
                )
                .arg(
                    Arg::with_name("out")
                        .help("sets output file")
                        .value_name("FILE")
                        .short("o")
                        .long("output"),
                )
                .setting(clap::AppSettings::ColorAuto)
                .setting(clap::AppSettings::ColoredHelp),
        )
        .setting(clap::AppSettings::ColorAuto)
        .setting(clap::AppSettings::ColoredHelp);

    let matches = app.clone().get_matches();

    // run subcommands
    if let Some(matches) = matches.subcommand_matches("completion") {
        completion(&matches, &mut app);
        std::process::exit(0);
    }
    drop(app);

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config = matches.value_of("config").unwrap_or("config.yaml");
    println!("Value for config: {}", config);

    let config: Option<serde_yaml::Value> = match std::fs::read_to_string(config) {
        Ok(config) => match serde_yaml::from_str(&config)  {
            Ok(config) => Some(config),
            Err(err) => {
                eprintln!("Error parsing config file: {}", err);
                None
            }
        },
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            None
        }
    };

    let mut conf = timedumper::Config::new();

    // read verbose value
    conf.verbose = {
        let mut a = matches.occurrences_of("verbose") as u8;
        let mut b = 0;
        if let Some(config) = &config {
            b = match config.get("verbose") {
                Some(verbose) => match verbose.as_i64() {
                    Some(verbose) => verbose as u8,
                    None => 0,
                },
                None => 0 as u8,
            };
            println!("{}", b);
        }
        if b > a {
            a = b;
        }
        a
    };

    conf.port = {
        let a: u16;
        if let Some(path) = &matches.value_of("port") {
            a = match path.parse() {
                Ok(port) => port,
                Err(err) => {
                    eprintln!("cannot parse {} to port: {}", path, err);
                    conf.port
                }
            };
        } else if let Some(path) = &config {
            a = match path.get("port") {
                Some(path) => match path.as_u64() {
                    Some(path) => path as u16,
                    None => conf.port,
                },
                None => conf.port,
            };
        } else {
            a = conf.port;
        }
        a
    };

    conf.updates = {
        let a: usize;
        if let Some(path) = &matches.value_of("updates") {
            a = match path.parse() {
                Ok(port) => port,
                Err(err) => {
                    eprintln!("cannot parse {} to updates count: {}", path, err);
                    conf.updates
                }
            };
        } else if let Some(path) = &config {
            a = match path.get("updates") {
                Some(path) => match path.as_u64() {
                    Some(path) => path as usize,
                    None => conf.updates,
                },
                None => conf.updates,
            };
        } else {
            a = conf.updates;
        }
        a
    };

    if let Some(name) = &matches.value_of("broker") {
        conf.broker = name.to_string();
    } 
    conf.broker = {
        let mut a: String;
        if let Some(path) = &matches.value_of("broker") {
            a = path.to_string();
        } else if let Some(path) = &config {
            a = match path.get("broker") {
                Some(path) => match path.as_str() {
                    Some(path) => path.to_string(),
                    None => conf.broker,
                },
                None => conf.broker,
            };
        } else {
            a = conf.broker;
        }
        a
    };

    if let Some(topic) = &matches.value_of("user") {
        conf.username = Some(topic.to_string());
    } else if let Some(config) = &config {
        if let Some(topic) = config.get("user") {
            if let Some(topic) = topic.as_str() {
                conf.password = Some(topic.to_string());
            }
        }
    }

    if let Some(topic) = &matches.value_of("pass") {
        conf.password = Some(topic.to_string());
    } else if let Some(config) = &config {
        if let Some(topic) = config.get("pass") {
            if let Some(topic) = topic.as_str() {
                conf.username = Some(topic.to_string());
            }
        }
    }

    if let Some(topic) = &matches.value_of("topic") {
        conf.topic = topic.to_string();
    } else if let Some(config) = &config {
        if let Some(topic) = config.get("topic") {
            if let Some(topic) = topic.as_str() {
                conf.topic = topic.to_string();
            }
        }
    }
    

    if conf.verbose >= 1 {
        println!("Debug{}: enabled", conf.verbose);
    }
    

    conf.run().unwrap();
}

// create completion
fn completion(args: &clap::ArgMatches, app: &mut App) {
    let shell: String = match args.value_of("shell") {
        Some(shell) => shell.to_string(),
        None => {
            let shell = match std::env::var("SHELL") {
                Ok(shell) => shell,
                Err(_) => "/bin/bash".to_string(),
            };
            let shell = std::path::Path::new(&shell);
            match shell.file_name() {
                Some(shell) => shell.to_os_string().to_string_lossy().to_string(),
                None => "bash".to_string(),
            }
        }
    };

    use clap::Shell;
    let shell_l = shell.to_lowercase();
    let shell: Shell;
    if shell_l == "fish".to_string() {
        shell = Shell::Fish;
    } else if shell_l == "zsh".to_string() {
        shell = Shell::Zsh;
    } else if shell_l == "powershell".to_string() {
        shell = Shell::PowerShell;
    } else if shell_l == "elvish".to_string() {
        shell = Shell::Elvish;
    } else {
        shell = Shell::Bash;
    }

    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write;

    let mut path = BufWriter::new(match args.value_of("out") {
        Some(x) => Box::new(
            File::create(&std::path::Path::new(x)).unwrap_or_else(|err| {
                eprintln!("Error opening file: {}", err);
                std::process::exit(1);
            }),
        ) as Box<Write>,
        None => Box::new(std::io::stdout()) as Box<Write>,
    });

    app.gen_completions_to("raspi_firmware", shell, &mut path);
}
