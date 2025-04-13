export enum ReviewStatus {
  Accepted = "Accepted",
  Rejected = "Rejected",
  Dropped = "Dropped",
  Ready = "Ready",
  UnMatched = "UnMatched",
}

export interface Anime {
  anilist_id: number
  year: number
  titles: string[]
  mappings: Mapping[]
}

export enum Platform {
  BgmTv = "BgmTv",
  Tmdb = "Tmdb",
}

export interface Mapping {
  id: string | null
  platform: Platform
  review_status: ReviewStatus
  score: number
}

export interface PaginationParams {
  page: number
  page_size: number
  status?: ReviewStatus | null
  year?: number | null
  anilist_id?: number
}

export interface PaginatedResult<T> {
  data: T[]
  total: number
  page: number
  page_size: number
}

// 任务相关类型定义
export enum Provider {
  Xai = "xai",
  Deepseek = "deepseek",
  Gemini = "gemini",
  OpenAI = "openai",
}

export const ProviderModelMap = {
  [Provider.Xai]: "grok-3-beta",
  [Provider.Deepseek]: "deepseek-chat",
  [Provider.Gemini]: "gemini-2.0-flash",
  [Provider.OpenAI]: "gpt-4o-mini",
}

export enum JobStatus {
  Created = "Created",
  Running = "Running",
  Paused = "Paused",
  Completed = "Completed",
  Failed = "Failed",
}

export interface JobDetails {
  platform: Platform
  year: number
  provider: Provider
  model: string
  num_animes_to_match: number
  num_processed: number
  num_matched: number
  num_failed: number
  job_start_time: string
  status: JobStatus
  current_index: number
}

export interface Summary {
  total_animes: number
  total_tmdb_matched: number
  total_tmdb_unmatched: number
  total_tmdb_dropped: number
  total_bgmtv_matched: number
  total_bgmtv_unmatched: number
  total_bgmtv_dropped: number
}

// 年份统计相关类型
export interface YearStatistic {
  year: number
  total_animes: number
  tmdb_matched: number
  tmdb_unmatched: number
  tmdb_dropped: number
  bgmtv_matched: number
  bgmtv_unmatched: number
  bgmtv_dropped: number
}

export interface YearStatistics {
  statistics: YearStatistic[]
}