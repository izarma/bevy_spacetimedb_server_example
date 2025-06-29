// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]
#[sats(crate = __lib)]
pub(super) struct ReceivePlayerInputArgs {
    pub x: f32,
    pub y: f32,
}

impl From<ReceivePlayerInputArgs> for super::Reducer {
    fn from(args: ReceivePlayerInputArgs) -> Self {
        Self::ReceivePlayerInput {
            x: args.x,
            y: args.y,
        }
    }
}

impl __sdk::InModule for ReceivePlayerInputArgs {
    type Module = super::RemoteModule;
}

pub struct ReceivePlayerInputCallbackId(__sdk::CallbackId);

#[allow(non_camel_case_types)]
/// Extension trait for access to the reducer `receive_player_input`.
///
/// Implemented for [`super::RemoteReducers`].
pub trait receive_player_input {
    /// Request that the remote module invoke the reducer `receive_player_input` to run as soon as possible.
    ///
    /// This method returns immediately, and errors only if we are unable to send the request.
    /// The reducer will run asynchronously in the future,
    ///  and its status can be observed by listening for [`Self::on_receive_player_input`] callbacks.
    fn receive_player_input(&self, x: f32, y: f32) -> __sdk::Result<()>;
    /// Register a callback to run whenever we are notified of an invocation of the reducer `receive_player_input`.
    ///
    /// Callbacks should inspect the [`__sdk::ReducerEvent`] contained in the [`super::ReducerEventContext`]
    /// to determine the reducer's status.
    ///
    /// The returned [`ReceivePlayerInputCallbackId`] can be passed to [`Self::remove_on_receive_player_input`]
    /// to cancel the callback.
    fn on_receive_player_input(
        &self,
        callback: impl FnMut(&super::ReducerEventContext, &f32, &f32) + Send + 'static,
    ) -> ReceivePlayerInputCallbackId;
    /// Cancel a callback previously registered by [`Self::on_receive_player_input`],
    /// causing it not to run in the future.
    fn remove_on_receive_player_input(&self, callback: ReceivePlayerInputCallbackId);
}

impl receive_player_input for super::RemoteReducers {
    fn receive_player_input(&self, x: f32, y: f32) -> __sdk::Result<()> {
        self.imp
            .call_reducer("receive_player_input", ReceivePlayerInputArgs { x, y })
    }
    fn on_receive_player_input(
        &self,
        mut callback: impl FnMut(&super::ReducerEventContext, &f32, &f32) + Send + 'static,
    ) -> ReceivePlayerInputCallbackId {
        ReceivePlayerInputCallbackId(self.imp.on_reducer(
            "receive_player_input",
            Box::new(move |ctx: &super::ReducerEventContext| {
                let super::ReducerEventContext {
                    event:
                        __sdk::ReducerEvent {
                            reducer: super::Reducer::ReceivePlayerInput { x, y },
                            ..
                        },
                    ..
                } = ctx
                else {
                    unreachable!()
                };
                callback(ctx, x, y)
            }),
        ))
    }
    fn remove_on_receive_player_input(&self, callback: ReceivePlayerInputCallbackId) {
        self.imp
            .remove_on_reducer("receive_player_input", callback.0)
    }
}

#[allow(non_camel_case_types)]
#[doc(hidden)]
/// Extension trait for setting the call-flags for the reducer `receive_player_input`.
///
/// Implemented for [`super::SetReducerFlags`].
///
/// This type is currently unstable and may be removed without a major version bump.
pub trait set_flags_for_receive_player_input {
    /// Set the call-reducer flags for the reducer `receive_player_input` to `flags`.
    ///
    /// This type is currently unstable and may be removed without a major version bump.
    fn receive_player_input(&self, flags: __ws::CallReducerFlags);
}

impl set_flags_for_receive_player_input for super::SetReducerFlags {
    fn receive_player_input(&self, flags: __ws::CallReducerFlags) {
        self.imp
            .set_call_reducer_flags("receive_player_input", flags);
    }
}
