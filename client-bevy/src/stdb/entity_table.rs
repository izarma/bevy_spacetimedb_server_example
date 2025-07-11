// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use super::entity_type::Entity;
use super::vec_2_type::Vec2;
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

/// Table handle for the table `entity`.
///
/// Obtain a handle from the [`EntityTableAccess::entity`] method on [`super::RemoteTables`],
/// like `ctx.db.entity()`.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.entity().on_insert(...)`.
pub struct EntityTableHandle<'ctx> {
    imp: __sdk::TableHandle<Entity>,
    ctx: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

#[allow(non_camel_case_types)]
/// Extension trait for access to the table `entity`.
///
/// Implemented for [`super::RemoteTables`].
pub trait EntityTableAccess {
    #[allow(non_snake_case)]
    /// Obtain a [`EntityTableHandle`], which mediates access to the table `entity`.
    fn entity(&self) -> EntityTableHandle<'_>;
}

impl EntityTableAccess for super::RemoteTables {
    fn entity(&self) -> EntityTableHandle<'_> {
        EntityTableHandle {
            imp: self.imp.get_table::<Entity>("entity"),
            ctx: std::marker::PhantomData,
        }
    }
}

pub struct EntityInsertCallbackId(__sdk::CallbackId);
pub struct EntityDeleteCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::Table for EntityTableHandle<'ctx> {
    type Row = Entity;
    type EventContext = super::EventContext;

    fn count(&self) -> u64 {
        self.imp.count()
    }
    fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.imp.iter()
    }

    type InsertCallbackId = EntityInsertCallbackId;

    fn on_insert(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> EntityInsertCallbackId {
        EntityInsertCallbackId(self.imp.on_insert(Box::new(callback)))
    }

    fn remove_on_insert(&self, callback: EntityInsertCallbackId) {
        self.imp.remove_on_insert(callback.0)
    }

    type DeleteCallbackId = EntityDeleteCallbackId;

    fn on_delete(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> EntityDeleteCallbackId {
        EntityDeleteCallbackId(self.imp.on_delete(Box::new(callback)))
    }

    fn remove_on_delete(&self, callback: EntityDeleteCallbackId) {
        self.imp.remove_on_delete(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn register_table(client_cache: &mut __sdk::ClientCache<super::RemoteModule>) {
    let _table = client_cache.get_or_make_table::<Entity>("entity");
    _table.add_unique_constraint::<u32>("entity_id", |row| &row.entity_id);
}
pub struct EntityUpdateCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::TableWithPrimaryKey for EntityTableHandle<'ctx> {
    type UpdateCallbackId = EntityUpdateCallbackId;

    fn on_update(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row, &Self::Row) + Send + 'static,
    ) -> EntityUpdateCallbackId {
        EntityUpdateCallbackId(self.imp.on_update(Box::new(callback)))
    }

    fn remove_on_update(&self, callback: EntityUpdateCallbackId) {
        self.imp.remove_on_update(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn parse_table_update(
    raw_updates: __ws::TableUpdate<__ws::BsatnFormat>,
) -> __sdk::Result<__sdk::TableUpdate<Entity>> {
    __sdk::TableUpdate::parse_table_update(raw_updates).map_err(|e| {
        __sdk::InternalError::failed_parse("TableUpdate<Entity>", "TableUpdate")
            .with_cause(e)
            .into()
    })
}

/// Access to the `entity_id` unique index on the table `entity`,
/// which allows point queries on the field of the same name
/// via the [`EntityEntityIdUnique::find`] method.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.entity().entity_id().find(...)`.
pub struct EntityEntityIdUnique<'ctx> {
    imp: __sdk::UniqueConstraintHandle<Entity, u32>,
    phantom: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

impl<'ctx> EntityTableHandle<'ctx> {
    /// Get a handle on the `entity_id` unique index on the table `entity`.
    pub fn entity_id(&self) -> EntityEntityIdUnique<'ctx> {
        EntityEntityIdUnique {
            imp: self.imp.get_unique_constraint::<u32>("entity_id"),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'ctx> EntityEntityIdUnique<'ctx> {
    /// Find the subscribed row whose `entity_id` column value is equal to `col_val`,
    /// if such a row is present in the client cache.
    pub fn find(&self, col_val: &u32) -> Option<Entity> {
        self.imp.find(col_val)
    }
}
