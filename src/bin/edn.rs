use anyhow::Result;
use edn::parser::Parser;

fn main() -> Result<()> {
    // let str = "(defn sum [xs]
    //              (reduce + 0 xs))
    //            (println (sum [1 2 3 4 5]))";

    let data = r#"{ 
    "string" "John"
    "int" 43
    "vec" ["+43 1234567" "+44 2345678"] 
    "map" {:key "val"}

    "mapns" #:ns{:key "val"}
    "inst" #inst "1985-04-12T23:20:50.52Z"
}"#;

    let mut parser = Parser::new(data);
    println!("{:?}", parser.read());
    println!("{:?}", parser.read());

    Ok(())
}
