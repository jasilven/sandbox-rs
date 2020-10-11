use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Person<'a> {
    first_name: &'a str,
    last_name: &'a str,
    age: u32,
}

fn main() -> Result<()> {
    let nc = nats::connect("localhost")?;

    let msg = "Hello!";
    nc.publish("foo", &msg)?;
    println!("published msg: {}", msg);

    let p = Person {
        first_name: "derek",
        last_name: "collison",
        age: 22,
    };

    let json = serde_json::to_vec(&p)?;
    nc.publish("foo", &json)?;
    println!("published json-msg: {:?}", &p);

    // Publish a request manually.
    let reply = nc.new_inbox();
    let subs = nc.subscribe("foo")?;
    nc.publish_request("foo", &reply, "Help me!")?;

    for msg in subs.messages() {
        println!("got msg: {}", msg);
    }

    Ok(())
}
