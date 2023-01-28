use std::collections::HashMap;
use tokio::sync::mpsc;
use super::Direction;
use tracing::{trace, debug, info, warn, error};
use async_trait::async_trait;
use movement::*;

pub mod movement {
    use std::fmt::Debug;
    use async_trait::async_trait;
    use tokio::sync::mpsc::error::SendError;
    use thiserror::Error;
    use super::Direction;

    #[async_trait]
    pub trait OrderMove: Send + Sync + Debug {
        async fn issue_move(&self, direction: Direction) -> Result<(), OrderError>;
    }

    #[derive(Error, Debug)]
    pub enum OrderError {
        #[error("Unable to issue new movement command `{0}`")]
        IssueMovement(String)
    }

    impl<T: Debug> From<SendError<T>> for OrderError {
        fn from(send_err: SendError<T>) -> Self {
            Self::IssueMovement(send_err.to_string())
        }
    }
}



#[derive(Debug)]
pub struct MoveCommandReceiver {
    command_rx: mpsc::Receiver<Direction>,
}

impl From<mpsc::Receiver<Direction>> for MoveCommandReceiver {
    fn from(command_rx: mpsc::Receiver<Direction>) -> Self {
        Self { command_rx }
    }
}

impl MoveCommandReceiver {
    pub async fn wait_for_command_and_act(&mut self, direction_command_counters: &mut HashMap<Direction, u32>, current_direction: &Direction) {
        match self.command_rx.recv().await {
            Some(c) => {
                if c == current_direction.opposite() {
                    return
                }
                direction_command_counters.entry(c).and_modify(|counter| *counter +=  1).or_insert(1);
                debug!("Received a move command from user {:?}", c);
            }
            None => {
                warn!("Received a move command although it was empty");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoveCommandIssuer {
    command_sender: mpsc::Sender<Direction>,
}

impl MoveCommandIssuer {
    pub fn set_issuer(&mut self, issuer: mpsc::Sender<Direction>) {
        self.command_sender = issuer;
    }
}

impl From<mpsc::Sender<Direction>> for MoveCommandIssuer {
    fn from(command_sender: mpsc::Sender<Direction>) -> Self {
        Self { command_sender }
    }
}

#[async_trait]
impl OrderMove for MoveCommandIssuer {
    #[tracing::instrument]
    async fn issue_move(&self, direction: Direction) -> Result<(), OrderError> {
        trace!("Issueing new move: {:?}", direction);
        self.command_sender
            .send(direction)
            .await?;

        Ok(())
    }
}
