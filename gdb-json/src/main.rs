use std::io::{self, BufRead};

use anyhow::Context;
use gdbmi::{
    parser::{Message, Response},
    raw::{Dict, GeneralMessage},
};
use serde_json::json;
use std::io::Write;

fn gdb_to_json(v: gdbmi::raw::Value) -> serde_json::Value {
    match v {
        gdbmi::raw::Value::String(s) => s.into(),
        gdbmi::raw::Value::List(l) => l.into_iter().map(gdb_to_json).collect(),
        gdbmi::raw::Value::Dict(d) => d.0.into_iter().map(|(k, v)| (k, gdb_to_json(v))).collect(),
    }
}

fn gdb_token_to_json(t: gdbmi::Token) -> serde_json::Value {
    t.0.into()
}
fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut stdin = stdin.lock();
    let mut stdout = stdout.lock();

    let mut buf = String::new();
    while stdin.read_line(&mut buf).context("read input")? != 0 {
        let msg = gdbmi::parser::parse_message(&buf)
            .with_context(|| format!("parsing message {buf:?}"))?;
        buf.clear();

        let msg = match msg {
            Message::Response(resp) => match resp {
                Response::Notify {
                    token,
                    message,
                    payload,
                } => {
                    json!({
                        "type": "notify",
                        "token": token.map(gdb_token_to_json),
                        "message": message,
                        "payload": gdb_to_json(gdbmi::raw::Value::Dict(payload)),
                    })
                }
                Response::Result {
                    token,
                    message,
                    payload,
                } => {
                    json!({
                        "type": "result",
                        "token": token.map(gdb_token_to_json),
                        "message": message,
                        "payload": payload.map(|x| gdb_to_json(gdbmi::raw::Value::Dict(x))).unwrap_or(serde_json::Value::Null),
                    })
                }
            },
            Message::General(g) => match g {
                GeneralMessage::Console(message) => json!({
                    "type": "console",
                    "message": message,
                }),
                GeneralMessage::Log(message) => json!({
                    "type": "log",
                    "message": message,
                }),
                GeneralMessage::Target(message) => json!({
                    "type": "target",
                    "message": message,
                }),
                GeneralMessage::Done => json!({"type": "done"}),
                GeneralMessage::InferiorStdout(message) => json!({
                    "type": "stdout",
                    "message": message,
                }),
                GeneralMessage::InferiorStderr(message) => json!({
                    "type": "stderr",
                    "message": message,
                }),
            },
        };

        serde_json::to_writer(&mut stdout, &msg).context("write message")?;
        writeln!(stdout)?;
    }
    Ok(())
}
