// shared fakes; each test module uses a different subset
#![allow(dead_code, unused_imports)]

pub mod fake_actor;
pub mod fake_block;
pub mod fake_block_type;
pub mod fake_chunk;
pub mod fake_client;
pub mod fake_connection;
pub mod fake_dimension;
pub mod fake_heap;
pub mod fake_level;
pub mod fake_palette;
pub mod fake_resolver;
pub mod fake_source;
pub mod fake_storage;
pub mod fake_sub_chunk;
pub mod fake_world;
pub mod heap_event;
pub mod recording_sink;
pub mod sink_call;
pub mod test_session;
pub mod vtable;

pub use fake_actor::FakeActor;
pub use fake_block::FakeBlock;
pub use fake_block_type::FakeBlockType;
pub use fake_chunk::FakeChunk;
pub use fake_client::FakeClient;
pub use fake_connection::FakeConnection;
pub use fake_dimension::FakeDimension;
pub use fake_heap::FakeHeap;
pub use fake_level::FakeLevel;
pub use fake_palette::FakePalette;
pub use fake_resolver::FakeResolver;
pub use fake_source::FakeSource;
pub use fake_storage::FakeStorage;
pub use fake_sub_chunk::FakeSubChunk;
pub use fake_world::FakeWorld;
pub use heap_event::HeapEvent;
pub use recording_sink::RecordingSink;
pub use sink_call::SinkCall;
pub use test_session::TestSession;
pub use vtable::Vtable;
