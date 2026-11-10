use std::collections::{HashMap, VecDeque};

use minimap_store::services::StoreService;
use minimap_store::structs::{JournalRecord, StoreCommand};

use crate::structs::JournalAck;

// records waiting for room in the store queue, and the acks expected for records already sent
pub struct JournalOutbox {
    queue: VecDeque<(Box<JournalRecord>, JournalAck)>,
    pending: HashMap<u64, JournalAck>,
    capacity: usize,
    next_token: u64,
    dropped: u64,
}

impl JournalOutbox {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity.min(1024)),
            pending: HashMap::new(),
            capacity: capacity.max(1),
            next_token: 1,
            dropped: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn pending_acks(&self) -> usize {
        self.pending.len()
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    // a full outbox drops its oldest record; the chunk stays journal-dirty and is written again later
    pub fn push(&mut self, record: JournalRecord, ack: JournalAck) {
        if self.queue.len() >= self.capacity && self.queue.pop_front().is_some() {
            self.dropped += 1;
            curia::warn!("journal outbox full: the oldest record is dropped");
        }
        self.queue.push_back((Box::new(record), ack));
    }

    // sends until the store queue is full
    pub fn flush(&mut self, store: &StoreService) {
        while let Some((record, ack)) = self.queue.pop_front() {
            let token = self.next_token;
            match store.try_send(StoreCommand::Append { record, token }) {
                Ok(()) => {
                    self.next_token += 1;
                    self.pending.insert(token, ack);
                }
                Err(StoreCommand::Append { record, .. }) => {
                    self.queue.push_front((record, ack));
                    break;
                }
                Err(_) => break,
            }
        }
        let window = 4 * self.capacity as u64;
        if self.pending.len() as u64 > window {
            let floor = self.next_token.saturating_sub(window);
            self.pending.retain(|token, _| *token >= floor);
        }
    }

    // everything still waiting, for a blocking flush when the world is left
    pub fn take_all(&mut self) -> Vec<Box<JournalRecord>> {
        self.queue.drain(..).map(|(record, _)| record).collect()
    }

    pub fn acknowledge(&mut self, token: u64) -> Option<JournalAck> {
        self.pending.remove(&token)
    }
}
