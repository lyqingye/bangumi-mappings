use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 animes 表
        manager
            .create_table(
                Table::create()
                    .table(Animes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Animes::AnilistId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Animes::MediaType).string().not_null())
                    .col(ColumnDef::new(Animes::Titles).text().not_null())
                    .col(ColumnDef::new(Animes::Year).integer())
                    .col(ColumnDef::new(Animes::Season).string())
                    .col(ColumnDef::new(Animes::StartDate).string())
                    .col(ColumnDef::new(Animes::EpisodeCount).integer())
                    .col(ColumnDef::new(Animes::SeasonNumber).integer())
                    .col(ColumnDef::new(Animes::EpisodeNumber).integer())
                    .col(ColumnDef::new(Animes::AbsoluteEpisodeNumber).integer())
                    .col(
                        ColumnDef::new(Animes::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Animes::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 anime_mappings 表
        manager
            .create_table(
                Table::create()
                    .table(Mappings::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Mappings::AnilistId).integer().not_null())
                    .col(ColumnDef::new(Mappings::Platform).string().not_null())
                    .col(ColumnDef::new(Mappings::PlatformId).string())
                    .col(
                        ColumnDef::new(Mappings::ReviewStatus)
                            .string()
                            .not_null()
                            .default("Unmapped"),
                    )
                    .col(
                        ColumnDef::new(Mappings::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Mappings::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Mappings::Score).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(Mappings::AnilistId)
                            .col(Mappings::Platform),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Mappings::Table, Mappings::AnilistId)
                            .to(Animes::Table, Animes::AnilistId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_animes_anilist_id")
                    .table(Animes::Table)
                    .col(Animes::AnilistId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_anime_mappings_anime_id")
                    .table(Mappings::Table)
                    .col(Mappings::AnilistId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_anime_mappings_platform_id")
                    .table(Mappings::Table)
                    .col(Mappings::PlatformId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_anime_mappings_status")
                    .table(Mappings::Table)
                    .col(Mappings::ReviewStatus)
                    .to_owned(),
            )
            .await?;

        // 创建唯一约束
        manager
            .create_index(
                Index::create()
                    .name("idx_anime_mappings_unique")
                    .table(Mappings::Table)
                    .col(Mappings::AnilistId)
                    .col(Mappings::Platform)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mappings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Animes::Table).to_owned())
            .await?;

        Ok(())
    }
}

// 表定义
#[derive(Iden)]
enum Animes {
    Table,
    AnilistId,
    MediaType,
    Titles,
    Year,
    Season,
    StartDate,
    EpisodeCount,
    SeasonNumber,
    EpisodeNumber,
    AbsoluteEpisodeNumber,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Mappings {
    Table,
    AnilistId,
    Platform,
    PlatformId,
    ReviewStatus,
    CreatedAt,
    UpdatedAt,
    Score,
}
