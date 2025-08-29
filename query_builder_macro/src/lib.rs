use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

/// Derive macro to generate a CodeIgniter3-style query builder for Sea-ORM entities
#[proc_macro_derive(QueryBuilder)]
pub fn derive_query_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let model_name = &input.ident;
    let entity_name = quote::format_ident!("Entity");
    let builder_name = quote::format_ident!("{}QueryBuilder", model_name);
    
    // Extract field information from the struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    input,
                    "QueryBuilder can only be derived for structs with named fields"
                ).to_compile_error().into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                input,
                "QueryBuilder can only be derived for structs"
            ).to_compile_error().into();
        }
    };

    // Generate column name constants for type safety
    let column_constants: Vec<_> = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let const_name = quote::format_ident!("{}", field_name.to_string().to_uppercase());
        quote! {
            pub const #const_name: &'static str = stringify!(#field_name);
        }
    }).collect();

    // Generate the query builder implementation
    let expanded = quote! {
        use sea_orm::{QuerySelect, QueryOrder};
        
        impl #entity_name {
            /// Create a new query builder for this entity
            pub fn query() -> #builder_name {
                #builder_name::new()
            }
        }

        /// CodeIgniter3-style query builder for #entity_name
        pub struct #builder_name {
            conditions: sea_orm::Condition,
            select_fields: Vec<sea_orm::sea_query::SimpleExpr>,
            order_by: Vec<(sea_orm::sea_query::SimpleExpr, sea_orm::Order)>,
            limit_value: Option<u64>,
            offset_value: Option<u64>,
        }

        impl #builder_name {
            /// Create a new query builder instance
            pub fn new() -> Self {
                Self {
                    conditions: sea_orm::Condition::all(),
                    select_fields: Vec::new(),
                    order_by: Vec::new(),
                    limit_value: None,
                    offset_value: None,
                }
            }

            /// Select specific fields (similar to CI3's select())
            pub fn select(mut self, column: Column) -> Self {
                self.select_fields.push(sea_orm::sea_query::Expr::col(column).into());
                self
            }

            /// Add a WHERE condition (similar to CI3's where())
            pub fn where_eq(mut self, column: Column, value: impl Into<sea_orm::Value>) -> Self {
                self.conditions = self.conditions.add(column.eq(value));
                self
            }

            /// Add a WHERE IN condition (similar to CI3's where_in())
            pub fn where_in<V: Into<sea_orm::Value>>(mut self, column: Column, values: Vec<V>) -> Self {
                let values: Vec<sea_orm::Value> = values.into_iter().map(|v| v.into()).collect();
                self.conditions = self.conditions.add(column.is_in(values));
                self
            }

            /// Add a LIKE condition (similar to CI3's like())
            pub fn like(mut self, column: Column, pattern: impl Into<String>) -> Self {
                let pattern_str = format!("%{}%", pattern.into());
                self.conditions = self.conditions.add(column.like(&pattern_str));
                self
            }

            /// Add a WHERE NOT NULL condition
            pub fn where_not_null(mut self, column: Column) -> Self {
                self.conditions = self.conditions.add(column.is_not_null());
                self
            }

            /// Add a WHERE NULL condition
            pub fn where_null(mut self, column: Column) -> Self {
                self.conditions = self.conditions.add(column.is_null());
                self
            }

            /// Add an OR condition
            pub fn or_where_eq(mut self, column: Column, value: impl Into<sea_orm::Value>) -> Self {
                self.conditions = sea_orm::Condition::any()
                    .add(self.conditions)
                    .add(column.eq(value));
                self
            }

            /// Add an ORDER BY clause (similar to CI3's order_by())
            pub fn order_by(mut self, column: Column, direction: sea_orm::Order) -> Self {
                self.order_by.push((sea_orm::sea_query::Expr::col(column).into(), direction));
                self
            }

            /// Order by ascending (convenience method)
            pub fn order_asc(mut self, column: Column) -> Self {
                self.order_by(column, sea_orm::Order::Asc)
            }

            /// Order by descending (convenience method)
            pub fn order_desc(mut self, column: Column) -> Self {
                self.order_by(column, sea_orm::Order::Desc)
            }

            /// Add a LIMIT clause (similar to CI3's limit())
            pub fn limit(mut self, limit: u64) -> Self {
                self.limit_value = Some(limit);
                self
            }

            /// Add an OFFSET clause (similar to CI3's offset())
            pub fn offset(mut self, offset: u64) -> Self {
                self.offset_value = Some(offset);
                self
            }

            /// Execute the query and return all results (similar to CI3's get())
            pub async fn get(self, db: &sea_orm::DbConn) -> Result<Vec<#model_name>, sea_orm::DbErr> {
                let mut query = #entity_name::find();

                // Apply WHERE conditions
                query = query.filter(self.conditions);

                // Apply SELECT fields (simplified - full table select for now)
                // In a more advanced version, we would support column selection
                // if !self.select_fields.is_empty() {
                //     query = query.select_only();
                //     for field in self.select_fields {
                //         query = query.column(field);
                //     }
                // }

                // Apply ORDER BY
                for (column, order) in self.order_by {
                    query = query.order_by(column, order);
                }

                // Apply LIMIT
                if let Some(limit) = self.limit_value {
                    query = query.limit(limit);
                }

                // Apply OFFSET
                if let Some(offset) = self.offset_value {
                    query = query.offset(offset);
                }

                query.all(db).await
            }

            /// Execute the query and return the first result (similar to CI3's first())
            pub async fn first(self, db: &sea_orm::DbConn) -> Result<Option<#model_name>, sea_orm::DbErr> {
                let mut query = #entity_name::find();

                // Apply WHERE conditions
                query = query.filter(self.conditions);

                // Apply SELECT fields (simplified - full table select for now)
                // In a more advanced version, we would support column selection
                // if !self.select_fields.is_empty() {
                //     query = query.select_only();
                //     for field in self.select_fields {
                //         query = query.column(field);
                //     }
                // }

                // Apply ORDER BY
                for (column, order) in self.order_by {
                    query = query.order_by(column, order);
                }

                query.one(db).await
            }

            /// Count the number of records (similar to CI3's count_all_results())
            pub async fn count(self, db: &sea_orm::DbConn) -> Result<u64, sea_orm::DbErr> {
                let mut query = #entity_name::find();

                // Apply WHERE conditions
                query = query.filter(self.conditions);

                query.count(db).await
            }
        }

        /// Column constants for type-safe field references
        impl #builder_name {
            #(#column_constants)*
        }
    };

    TokenStream::from(expanded)
}