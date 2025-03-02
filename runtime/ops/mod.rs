// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

pub mod crypto;
pub mod fetch;
pub mod fs;
pub mod fs_events;
pub mod io;
pub mod net;
#[cfg(unix)]
mod net_unix;
mod ops_buffer;
pub mod os;
pub mod permissions;
pub mod plugin;
pub mod process;
pub mod runtime;
pub mod signal;
pub mod timers;
pub mod tls;
pub mod tty;
pub mod url;
pub mod web_worker;
pub mod webgpu;
pub mod websocket;
pub mod worker_host;

use crate::metrics::metrics_op;
use deno_core::error::AnyError;
use deno_core::json_op_async;
use deno_core::json_op_sync;
use deno_core::serde::de::DeserializeOwned;
use deno_core::serde::Serialize;
use deno_core::BufVec;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::ZeroCopyBuf;
use ops_buffer::buffer_op_async;
use ops_buffer::buffer_op_sync;
use ops_buffer::ValueOrVector;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;

pub fn reg_json_async<F, V, R, RV>(
  rt: &mut JsRuntime,
  name: &'static str,
  op_fn: F,
) where
  F: Fn(Rc<RefCell<OpState>>, V, BufVec) -> R + 'static,
  V: DeserializeOwned,
  R: Future<Output = Result<RV, AnyError>> + 'static,
  RV: Serialize,
{
  rt.register_op(name, metrics_op(name, json_op_async(op_fn)));
}

pub fn reg_json_sync<F, V, R>(rt: &mut JsRuntime, name: &'static str, op_fn: F)
where
  F: Fn(&mut OpState, V, &mut [ZeroCopyBuf]) -> Result<R, AnyError> + 'static,
  V: DeserializeOwned,
  R: Serialize,
{
  rt.register_op(name, metrics_op(name, json_op_sync(op_fn)));
}

pub fn reg_buffer_async<F, R, RV>(
  rt: &mut JsRuntime,
  name: &'static str,
  op_fn: F,
) where
  F: Fn(Rc<RefCell<OpState>>, u32, BufVec) -> R + 'static,
  R: Future<Output = Result<RV, AnyError>> + 'static,
  RV: ValueOrVector,
{
  rt.register_op(name, metrics_op(name, buffer_op_async(op_fn)));
}

pub fn reg_buffer_sync<F, R>(rt: &mut JsRuntime, name: &'static str, op_fn: F)
where
  F: Fn(&mut OpState, u32, &mut [ZeroCopyBuf]) -> Result<R, AnyError> + 'static,
  R: ValueOrVector,
{
  rt.register_op(name, metrics_op(name, buffer_op_sync(op_fn)));
}

/// `UnstableChecker` is a struct so it can be placed inside `GothamState`;
/// using type alias for a bool could work, but there's a high chance
/// that there might be another type alias pointing to a bool, which
/// would override previously used alias.
pub struct UnstableChecker {
  pub unstable: bool,
}

impl UnstableChecker {
  /// Quits the process if the --unstable flag was not provided.
  ///
  /// This is intentionally a non-recoverable check so that people cannot probe
  /// for unstable APIs from stable programs.
  // NOTE(bartlomieju): keep in sync with `cli/program_state.rs`
  pub fn check_unstable(&self, api_name: &str) {
    if !self.unstable {
      eprintln!(
        "Unstable API '{}'. The --unstable flag must be provided.",
        api_name
      );
      std::process::exit(70);
    }
  }
}
/// Helper for checking unstable features. Used for sync ops.
pub fn check_unstable(state: &OpState, api_name: &str) {
  state.borrow::<UnstableChecker>().check_unstable(api_name)
}

/// Helper for checking unstable features. Used for async ops.
pub fn check_unstable2(state: &Rc<RefCell<OpState>>, api_name: &str) {
  let state = state.borrow();
  state.borrow::<UnstableChecker>().check_unstable(api_name)
}
