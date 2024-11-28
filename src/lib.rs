use mcap::{Channel, Schema, records::MessageHeader};

use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{Seek, Write};
use std::sync::{Arc, Mutex};

pub trait McapMessage<'a> {
    type Error;
    fn topic(self) -> &'static str;
    fn channel(self) -> Channel<'a>;
    fn message(&self) -> Result<Cow<'a, [u8]>, Self::Error>;
}

pub struct McapLogger<'a, W: Write + Seek> {
    writer: Arc<Mutex<mcap::Writer<'a, W>>>,
    headers: HashMap<&'a str, MessageHeader>,
}

impl<'a, W: Write + Seek> McapLogger<'a, W> {
    pub fn new(writer: mcap::Writer<'a, W>) -> Self {
        Self {
            writer: Arc::new(Mutex::new(writer)),
            headers: HashMap::new(),
        }
    }

    pub fn event<M: McapMessage<'a> + std::fmt::Debug + std::marker::Copy>(&mut self, level: tracing::Level, msg: &M)
    where
        <M as McapMessage<'a>>::Error: std::fmt::Debug
    {
        let mut w = self.writer.lock().unwrap();
        match level {
            // "MCAP event on topic {}: {:?}"
            _ => {
                if let Some(mut header) = self.headers.get_mut(&msg.topic()) {
                    header.sequence += 1;
                    header.log_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64;
                    header.publish_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64;
                    
                    w.write_to_known_channel(header, &msg.message().unwrap()).unwrap();
                } else {
                    let channel_id = w.add_channel(&msg.channel()).unwrap();
                    let mut header = MessageHeader {
                        channel_id,
                        sequence: 0,
                        log_time: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64,
                        publish_time: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64,
                    };
                    self.headers.insert(msg.topic(), header);
                    w.write_to_known_channel(&header, &msg.message().unwrap()).unwrap();
                }

                tracing::error!("MCAP message on topic: {} msg: {:?}", msg.topic(), msg);
            }
        }
    }

    pub fn close(&mut self) {
        let mut out = self.writer.lock().unwrap();
        out.finish().unwrap();
    }
}

