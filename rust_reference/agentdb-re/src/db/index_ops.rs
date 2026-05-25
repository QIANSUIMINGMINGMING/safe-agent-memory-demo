use crate::db::Database;
use crate::error::DbError;
use crate::index::{InMemoryVectorIndex, SemanticEmbedder, SemanticSearchHit};
use crate::judge::SemanticSubject;
use crate::row::RowId;

impl Database {
    pub fn rebuild_semantic_index<E: SemanticEmbedder>(
        &mut self,
        table_name: &str,
        embedder: &E,
    ) -> Result<(), DbError> {
        let entries = {
            let table = self.table(table_name)?;
            let mut entries = Vec::with_capacity(table.rows().len());

            for row in table.rows() {
                let subject = SemanticSubject::from_row(table.schema(), row)?;
                let vector = embedder.embed(&subject.canonical_text())?;
                entries.push((row.id(), vector));
            }

            entries
        };

        let index = InMemoryVectorIndex::build(entries)?;
        self.semantic_indexes.insert(table_name.to_owned(), index);
        Ok(())
    }

    pub fn semantic_index(&self, table_name: &str) -> Result<&InMemoryVectorIndex, DbError> {
        let _ = self.table(table_name)?;
        self.semantic_indexes
            .get(table_name)
            .ok_or_else(|| DbError::SemanticIndexNotBuilt(table_name.to_owned()))
    }

    pub fn search_semantic<E: SemanticEmbedder>(
        &self,
        table_name: &str,
        query_text: &str,
        limit: usize,
        embedder: &E,
    ) -> Result<Vec<SemanticSearchHit>, DbError> {
        let _ = self.table(table_name)?;
        let query = embedder.embed(query_text)?;
        self.semantic_index(table_name)?
            .search(&query, limit, None)
            .map_err(Into::into)
    }

    pub fn search_semantic_rows<E: SemanticEmbedder>(
        &self,
        table_name: &str,
        row_ids: &[RowId],
        query_text: &str,
        limit: usize,
        embedder: &E,
    ) -> Result<Vec<SemanticSearchHit>, DbError> {
        let _ = self.table(table_name)?;
        let query = embedder.embed(query_text)?;
        self.semantic_index(table_name)?
            .search_rows(&query, row_ids.iter().copied(), limit, None)
            .map_err(Into::into)
    }

    pub fn semantic_neighbors(
        &self,
        table_name: &str,
        row_id: RowId,
        limit: usize,
    ) -> Result<Vec<SemanticSearchHit>, DbError> {
        let table = self.table(table_name)?;
        if table.row(row_id).is_none() {
            return Err(DbError::RowNotFound {
                table: table_name.to_owned(),
                row_id,
            });
        }

        let index = self.semantic_index(table_name)?;
        let Some(query) = index.row_vector(row_id) else {
            return Err(DbError::SemanticIndexRowNotIndexed {
                table: table_name.to_owned(),
                row_id,
            });
        };

        index.search(query, limit, Some(row_id)).map_err(Into::into)
    }

    pub(crate) fn invalidate_semantic_index(&mut self, table_name: &str) {
        self.semantic_indexes.remove(table_name);
    }

    pub(crate) fn refresh_semantic_index_for_row<E: SemanticEmbedder>(
        &mut self,
        table_name: &str,
        row_id: RowId,
        embedder: &E,
    ) -> Result<(), DbError> {
        let subject = {
            let table = self.table(table_name)?;
            let row = table.row(row_id).ok_or_else(|| DbError::RowNotFound {
                table: table_name.to_owned(),
                row_id,
            })?;
            SemanticSubject::from_row(table.schema(), row)?
        };
        let vector = embedder.embed(&subject.canonical_text())?;

        if let Some(index) = self.semantic_indexes.get_mut(table_name) {
            index.insert(row_id, vector)?;
            Ok(())
        } else {
            self.rebuild_semantic_index(table_name, embedder)
        }
    }
}
