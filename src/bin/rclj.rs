use anyhow::{anyhow, bail, format_err, Result};
use edn::parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::boxed::Box;
use std::collections::{BTreeMap, HashMap};
use std::io::BufReader;
use std::io::{stderr, stdout, BufRead};
use std::io::{Read, Write};
use std::net::TcpStream;
use structopt::StructOpt;

fn write_and_flush(w: &mut dyn Write, data: &str) -> Result<()> {
    w.write_all(data.as_bytes())?;
    w.flush()?;

    Ok(())
}

fn get_value(key: &str, map: &BTreeMap<edn::Value, edn::Value>) -> Option<String> {
    match map.get(&edn::Value::Keyword(key.into())) {
        Some(edn::Value::String(value)) => {
            return Some(value.into());
        }
        Some(edn::Value::Keyword(value)) => {
            return Some(value.into());
        }
        Some(edn::Value::Boolean(value)) => {
            return Some(value.to_string());
        }
        _ => None,
    }
}

enum Response {
    Done(Option<String>),
    Exception(String),
    StdOut(String),
    StdErr(String),
    Other(String),
}

trait Repl {
    fn send(&mut self, s: &str) -> Result<()>;
    fn get_ns(&self) -> String;
    fn recv(&mut self) -> Result<Response>;
    fn quit(&mut self) -> Result<()>;
    fn repl_type(&self) -> String;
}

struct Nrepl {
    ns: String,
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl Nrepl {
    fn new(stream: TcpStream) -> Result<Nrepl> {
        let nrepl = Nrepl {
            ns: "user".into(),
            writer: stream.try_clone()?,
            reader: BufReader::new(stream),
        };
        Ok(nrepl)
    }
}

impl Repl for Nrepl {
    fn quit(&mut self) -> Result<()> {
        // write_and_flush(&mut self.writer, ":repl/quit\n")?;
        Ok(())
    }

    fn get_ns(&self) -> String {
        self.ns.to_string()
    }

    fn repl_type(&self) -> String {
        "nREPL".to_string()
    }

    fn send(&mut self, s: &str) -> Result<()> {
        let mut map: HashMap<&str, &str> = HashMap::new();

        map.insert("op", "eval");
        map.insert("code", s);

        write_and_flush(&mut self.writer, &bencode_rs::Value::from(map).to_bencode())?;

        Ok(())
    }

    fn recv(&mut self) -> Result<Response> {
        match bencode_rs::parse_bencode(&mut self.reader) {
            Ok(Some(bencode_rs::Value::Map(map))) => {
                if let Some(bencode_rs::Value::Str(s)) =
                    map.get(&bencode_rs::Value::Str("ns".into()))
                {
                    self.ns = s.into();
                }
                if let Some(bencode_rs::Value::Str(s)) =
                    map.get(&bencode_rs::Value::Str("err".into()))
                {
                    return Ok(Response::StdErr(s.into()));
                } else if let Some(bencode_rs::Value::Str(s)) =
                    map.get(&bencode_rs::Value::Str("out".into()))
                {
                    return Ok(Response::StdOut(s.into()));
                } else if let Some(bencode_rs::Value::Str(s)) =
                    map.get(&bencode_rs::Value::Str("value".into()))
                {
                    return Ok(Response::StdOut(format!("{}\n", s)));
                } else if let Some(bencode_rs::Value::List(list)) =
                    map.get(&bencode_rs::Value::Str("status".into()))
                {
                    if list.contains(&bencode_rs::Value::Str("done".into())) {
                        return Ok(Response::Done(None));
                    } else {
                        return Ok(Response::Other("".into()));
                    }
                } else {
                    bail!("Malformat response from nREPL: {:?}", map);
                }
            }
            Ok(None) => bail!("nREPL died?"),
            Ok(_) => bail!("Unexpected response from nREPL"),
            Err(e) => bail!("Error: {} ", e),
        }
    }
}

fn is_valid_form(s: &str) -> bool {
    let mut parser = edn::parser::Parser::new(s);
    match parser.read() {
        Some(Ok(_)) => true,
        _ => false,
    }
}

struct Prepl {
    ns: String,
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl Prepl {
    fn new(stream: TcpStream) -> Result<Prepl> {
        let prepl = Prepl {
            ns: "user".into(),
            writer: stream.try_clone()?,
            reader: BufReader::new(stream),
        };
        Ok(prepl)
    }
}

impl Repl for Prepl {
    fn quit(&mut self) -> Result<()> {
        write_and_flush(&mut self.writer, ":repl/quit\n")?;
        Ok(())
    }

    fn get_ns(&self) -> String {
        self.ns.to_string()
    }

    fn repl_type(&self) -> String {
        "pREPL".to_string()
    }

    fn send(&mut self, s: &str) -> Result<()> {
        write_and_flush(&mut self.writer, &s)?;
        Ok(())
    }

    fn recv(&mut self) -> Result<Response> {
        let mut buf = String::from("");
        self.reader.read_line(&mut buf)?;
        let mut parser = Parser::new(&buf);
        let response = parser
            .read()
            .ok_or(format_err!("Unexpected 'None'-response from pREPL"))?;

        match response {
            Ok(edn::Value::Map(map)) => {
                let val = get_value("val", &map).ok_or(anyhow!("'val' not found in response"))?;
                let tag = get_value("tag", &map).ok_or(anyhow!("'tag' not found in response"))?;
                self.ns = get_value("ns", &map).unwrap_or(self.get_ns());
                match tag.as_str() {
                    "err" => {
                        return Ok(Response::StdErr(val.into()));
                    }
                    "out" => {
                        return Ok(Response::StdOut(val.into()));
                    }
                    "ret" => {
                        if get_value("exception", &map).is_some() {
                            let mut parser = Parser::new(val.as_str());
                            if let Ok(edn::Value::Map(emap)) = parser
                                .read()
                                .ok_or(anyhow!("Unable to parse exception '{}'", val))?
                            {
                                match get_value("cause", &emap) {
                                    Some(cause) => return Ok(Response::Exception(cause)),
                                    None => bail!("Unable to parse error '{}'", val),
                                }
                            } else {
                                bail!("Unable to parse error '{}'", val);
                            }
                        } else {
                            return Ok(Response::Done(Some(val)));
                        }
                    }
                    _ => bail!("Unknown tag in response '{:?}'", tag),
                }
            }
            Ok(_) => bail!("Unexpected pREPL-response '{:?}'", response),
            Err(e) => bail!("Parse error: {}", e.message),
        }
    }
}

fn readline(namespace: &str) -> Result<Option<String>> {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(&format!("{}=> ", &namespace));
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                } else {
                    rl.add_history_entry(line);
                    return Ok(Some(line.into()));
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                return Ok(None);
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                return Ok(None);
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}

fn main_loop(mut repl: Box<dyn Repl>) -> Result<()> {
    let mut out = stdout();
    let mut err = stderr();

    loop {
        match readline(&repl.get_ns())? {
            Some(s) => {
                if repl.repl_type() == "pREPL" {
                    use std::panic;

                    let result = panic::catch_unwind(|| {
                        // This ugly pacic catching is needed for prepl, which is stream based and
                        // expects correctly formatted forms in one go. So before sending forms to
                        // prepl they need to be validated and the edn library used for validating
                        // occationally panics for invalid cases..
                        if !is_valid_form(&s) {
                            panic!();
                        }
                    });
                    if result.is_err() {
                        println!("Not a valid form '{}'", s);
                        continue;
                    }
                }
                repl.send(&s)?;
            }
            None => {
                repl.quit()?;
                break;
            }
        }
        loop {
            match repl.recv()? {
                Response::StdErr(s) => {
                    write_and_flush(&mut err, &s)?;
                }
                Response::StdOut(s) => {
                    write_and_flush(&mut out, &s)?;
                }
                Response::Exception(s) => {
                    write_and_flush(&mut out, &format!("{}\n", &s))?;
                    break;
                }
                Response::Other(_) => {}
                Response::Done(opt) => {
                    if let Some(s) = opt {
                        write_and_flush(&mut out, &format!("{}\n", &s))?;
                    }
                    break;
                }
            }
        }
    }

    Ok(())
}

fn get_repl(host: &str, port: usize) -> Result<Box<dyn Repl>> {
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))?;
    let _ = stream.write_all(b"d4:code7:(+ 1 1)2:op4:evale\n")?;
    stream.flush()?;

    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;

    // restart connection from clean state
    stream.shutdown(std::net::Shutdown::Both)?;
    stream = TcpStream::connect(format!("{}:{}", host, port))?;

    match buf[0] {
        123 => {
            let repl = Prepl::new(stream)?;
            return Ok(Box::new(repl));
        }
        100 => {
            let repl = Nrepl::new(stream)?;
            return Ok(Box::new(repl));
        }
        _ => bail!("Unable to identify nREPL/pREPL"),
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "rclj")]
struct Opt {
    /// Repl host
    #[structopt(short, default_value = "127.0.0.1")]
    host: String,

    /// Repl port
    #[structopt(short)]
    port: usize,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let repl = get_repl(&opt.host, opt.port)?;

    println!(
        "\nConnected to {} at {}:{}",
        repl.repl_type(),
        &opt.host,
        opt.port
    );
    println!("Exit: CTRL+D\n");

    main_loop(repl)?;

    Ok(())
}
