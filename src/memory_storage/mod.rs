use std::collections::BTreeMap;

use actix::prelude::*;
use log::{debug};
use serde::{Serialize, Deserialize};

use crate::{
    AppError, NodeId,
    messages,
    storage::{
        AppendLogEntries,
        ApplyEntriesToStateMachine,
        CreateSnapshot,
        CurrentSnapshotData,
        GetCurrentSnapshot,
        GetInitialState,
        GetLogEntries,
        HardState,
        InitialState,
        InstallSnapshot,
        RaftStorage,
        SaveHardState,
    },
};

/// The concrete error type used by the `MemoryStorage` system.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryStorageError;

impl AppError for MemoryStorageError {}

/// A concrete implementation of the `RaftStorage` trait.
///
/// This is primarity for testing and demo purposes. In a real application, storing Raft's data
/// on a stable storage medium is expected.
///
/// This storage implementation structures its data as an append-only immutable log. The contents
/// of the entries given to this storage implementation are not ready or manipulated.
pub struct MemoryStorage {
    hs: HardState,
    log: BTreeMap<u64, messages::Entry>,
    snapshot_data: Option<CurrentSnapshotData>,
    snapshot_dir: String,
    state_machine: BTreeMap<u64, messages::Entry>,
}

impl RaftStorage<MemoryStorageError> for MemoryStorage {
    /// Create a new instance.
    fn new(members: Vec<NodeId>, snapshot_dir: String) -> Self {
        Self{
            hs: HardState{current_term: 0, voted_for: None, members},
            log: Default::default(),
            snapshot_data: None, snapshot_dir,
            state_machine: Default::default(),
        }
    }
}

impl Actor for MemoryStorage {
    type Context = Context<Self>;

    /// Start this actor.
    fn started(&mut self, _ctx: &mut Self::Context) {}
}

impl Handler<GetInitialState<MemoryStorageError>> for MemoryStorage {
    type Result = ResponseActFuture<Self, InitialState, MemoryStorageError>;

    fn handle(&mut self, _: GetInitialState<MemoryStorageError>, _: &mut Self::Context) -> Self::Result {
        Box::new(fut::ok(InitialState{
            last_log_index: self.log.iter().last().map(|e| *e.0).unwrap_or(0),
            last_log_term: self.log.iter().last().map(|e| e.1.term).unwrap_or(0),
            last_applied_log: self.state_machine.iter().last().map(|e| *e.0).unwrap_or(0),
            hard_state: self.hs.clone(),
        }))
    }
}

impl Handler<SaveHardState<MemoryStorageError>> for MemoryStorage {
    type Result = ResponseActFuture<Self, (), MemoryStorageError>;

    fn handle(&mut self, msg: SaveHardState<MemoryStorageError>, _: &mut Self::Context) -> Self::Result {
        self.hs = msg.hs;
        Box::new(fut::ok(()))
    }
}

impl Handler<GetLogEntries<MemoryStorageError>> for MemoryStorage {
    type Result = ResponseActFuture<Self, Vec<messages::Entry>, MemoryStorageError>;

    fn handle(&mut self, msg: GetLogEntries<MemoryStorageError>, _: &mut Self::Context) -> Self::Result {
        Box::new(fut::ok(self.log.range(msg.start..msg.stop).map(|e| e.1.clone()).collect()))
    }
}

impl Handler<AppendLogEntries<MemoryStorageError>> for MemoryStorage {
    type Result = ResponseActFuture<Self, (), MemoryStorageError>;

    fn handle(&mut self, msg: AppendLogEntries<MemoryStorageError>, _: &mut Self::Context) -> Self::Result {
        msg.entries.iter().for_each(|e| {
            self.log.insert(e.index, e.clone());
        });
        Box::new(fut::ok(()))
    }
}

impl Handler<ApplyEntriesToStateMachine<MemoryStorageError>> for MemoryStorage {
    type Result = ResponseActFuture<Self, (), MemoryStorageError>;

    fn handle(&mut self, msg: ApplyEntriesToStateMachine<MemoryStorageError>, _ctx: &mut Self::Context) -> Self::Result {
        msg.entries.iter().for_each(|e| {
            self.state_machine.insert(e.index, e.clone());
        });
        Box::new(fut::ok(()))
    }
}

impl Handler<CreateSnapshot<MemoryStorageError>> for MemoryStorage {
    type Result = ResponseActFuture<Self, CurrentSnapshotData, MemoryStorageError>;

    fn handle(&mut self, _msg: CreateSnapshot<MemoryStorageError>, _: &mut Self::Context) -> Self::Result {
        debug!("Creating new snapshot in directory: {}", &self.snapshot_dir);
        Box::new(fut::err(MemoryStorageError))
    }
}

impl Handler<InstallSnapshot<MemoryStorageError>> for MemoryStorage {
    type Result = ResponseActFuture<Self, (), MemoryStorageError>;

    fn handle(&mut self, _msg: InstallSnapshot<MemoryStorageError>, _: &mut Self::Context) -> Self::Result {
        Box::new(fut::err(MemoryStorageError))
    }
}

impl Handler<GetCurrentSnapshot<MemoryStorageError>> for MemoryStorage {
    type Result = ResponseActFuture<Self, Option<CurrentSnapshotData>, MemoryStorageError>;

    fn handle(&mut self, _: GetCurrentSnapshot<MemoryStorageError>, _: &mut Self::Context) -> Self::Result {
        Box::new(fut::ok(self.snapshot_data.clone()))
    }
}
