use anyhow::Result;

use mcap::{Channel, Schema, records::MessageHeader, Writer};

use tracing_mcap::{McapMessage, McapLogger};

use tracing_subscriber::prelude::*;
use tracing_subscriber::{Registry, layer::SubscriberExt, util::SubscriberInitExt};

use schemars::{schema_for, JsonSchema};

use serde::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::io::BufWriter;

use valuable::Valuable;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct Pose {
    x: f32,
    y: f32,
    z: f32,
}

impl<'a> McapMessage<'a> for Pose {
    type Error = anyhow::Error;
    fn topic(self) -> &'static str {
        "pose"
    }

    fn channel(self) -> Channel<'a> {
        let schema = schema_for!(Self);
        let schema = serde_json::to_vec(&schema).unwrap();
        Channel {
            topic: String::from("pose"),
            schema: Some(Schema {
                name: String::from("Pose"),
                encoding: String::from("jsonschema"),
                data: Cow::Owned(schema),
            }.into()),
            message_encoding: String::from("json"),
            metadata: BTreeMap::default()
        }
    }

    fn message(&self) -> Result<Cow<'a, [u8]>, Self::Error> {
        let msg = serde_json::to_vec(self)?;
        Ok(Cow::Owned(msg))
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mcap_writer = Writer::new(
        BufWriter::new(fs::File::create("out.mcap").unwrap())
    ).unwrap();
    let mut mcap_logger = McapLogger::new(mcap_writer);
    let mut p = Pose {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    mcap_logger.event(tracing::Level::TRACE, &p);
    p.x = 1.5;
    mcap_logger.event(tracing::Level::TRACE, &p);
    mcap_logger.close();
    Ok(())
}
