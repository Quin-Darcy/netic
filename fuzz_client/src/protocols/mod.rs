mod protocol_trait;
mod greeting_protocol;
// mod your_protocol;

pub use protocol_trait::Protocol;

pub use greeting_protocol::{GreetingMessageType, GreetingMessageSectionsKey, GreetingMessageSectionsValue, GreetingProtocol};
// pub use your_protocol::{YourProtocolMessageType, YourProtocolMessageSectionsKey, YourProtocolMessageSectionsValue, YourProtocol};