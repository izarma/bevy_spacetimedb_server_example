// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use super::scheduled_tick_type::ScheduledTick;
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

/// Table handle for the table `scheduled_tick`.
///
/// Obtain a handle from the [`ScheduledTickTableAccess::scheduled_tick`] method on [`super::RemoteTables`],
/// like `ctx.db.scheduled_tick()`.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.scheduled_tick().on_insert(...)`.
pub struct ScheduledTickTableHandle<'ctx> {
    imp: __sdk::TableHandle<ScheduledTick>,
    ctx: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

#[allow(non_camel_case_types)]
/// Extension trait for access to the table `scheduled_tick`.
///
/// Implemented for [`super::RemoteTables`].
pub trait ScheduledTickTableAccess {
    #[allow(non_snake_case)]
    /// Obtain a [`ScheduledTickTableHandle`], which mediates access to the table `scheduled_tick`.
    fn scheduled_tick(&self) -> ScheduledTickTableHandle<'_>;
}

impl ScheduledTickTableAccess for super::RemoteTables {
    fn scheduled_tick(&self) -> ScheduledTickTableHandle<'_> {
        ScheduledTickTableHandle {
            imp: self.imp.get_table::<ScheduledTick>("scheduled_tick"),
            ctx: std::marker::PhantomData,
        }
    }
}

pub struct ScheduledTickInsertCallbackId(__sdk::CallbackId);
pub struct ScheduledTickDeleteCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::Table for ScheduledTickTableHandle<'ctx> {
    type Row = ScheduledTick;
    type EventContext = super::EventContext;

    fn count(&self) -> u64 {
        self.imp.count()
    }
    fn iter(&self) -> impl Iterator<Item = ScheduledTick> + '_ {
        self.imp.iter()
    }

    type InsertCallbackId = ScheduledTickInsertCallbackId;

    fn on_insert(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> ScheduledTickInsertCallbackId {
        ScheduledTickInsertCallbackId(self.imp.on_insert(Box::new(callback)))
    }

    fn remove_on_insert(&self, callback: ScheduledTickInsertCallbackId) {
        self.imp.remove_on_insert(callback.0)
    }

    type DeleteCallbackId = ScheduledTickDeleteCallbackId;

    fn on_delete(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> ScheduledTickDeleteCallbackId {
        ScheduledTickDeleteCallbackId(self.imp.on_delete(Box::new(callback)))
    }

    fn remove_on_delete(&self, callback: ScheduledTickDeleteCallbackId) {
        self.imp.remove_on_delete(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn register_table(client_cache: &mut __sdk::ClientCache<super::RemoteModule>) {
    let _table = client_cache.get_or_make_table::<ScheduledTick>("scheduled_tick");
    _table.add_unique_constraint::<u64>("scheduled_id", |row| &row.scheduled_id);
}
pub struct ScheduledTickUpdateCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::TableWithPrimaryKey for ScheduledTickTableHandle<'ctx> {
    type UpdateCallbackId = ScheduledTickUpdateCallbackId;

    fn on_update(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row, &Self::Row) + Send + 'static,
    ) -> ScheduledTickUpdateCallbackId {
        ScheduledTickUpdateCallbackId(self.imp.on_update(Box::new(callback)))
    }

    fn remove_on_update(&self, callback: ScheduledTickUpdateCallbackId) {
        self.imp.remove_on_update(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn parse_table_update(
    raw_updates: __ws::TableUpdate<__ws::BsatnFormat>,
) -> __sdk::Result<__sdk::TableUpdate<ScheduledTick>> {
    __sdk::TableUpdate::parse_table_update(raw_updates).map_err(|e| {
        __sdk::InternalError::failed_parse("TableUpdate<ScheduledTick>", "TableUpdate")
            .with_cause(e)
            .into()
    })
}

/// Access to the `scheduled_id` unique index on the table `scheduled_tick`,
/// which allows point queries on the field of the same name
/// via the [`ScheduledTickScheduledIdUnique::find`] method.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.scheduled_tick().scheduled_id().find(...)`.
pub struct ScheduledTickScheduledIdUnique<'ctx> {
    imp: __sdk::UniqueConstraintHandle<ScheduledTick, u64>,
    phantom: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

impl<'ctx> ScheduledTickTableHandle<'ctx> {
    /// Get a handle on the `scheduled_id` unique index on the table `scheduled_tick`.
    pub fn scheduled_id(&self) -> ScheduledTickScheduledIdUnique<'ctx> {
        ScheduledTickScheduledIdUnique {
            imp: self.imp.get_unique_constraint::<u64>("scheduled_id"),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'ctx> ScheduledTickScheduledIdUnique<'ctx> {
    /// Find the subscribed row whose `scheduled_id` column value is equal to `col_val`,
    /// if such a row is present in the client cache.
    pub fn find(&self, col_val: &u64) -> Option<ScheduledTick> {
        self.imp.find(col_val)
    }
}
