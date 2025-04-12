import { apiClient } from "./api-client"
import type { Anime, JobDetails, PaginatedResult, PaginationParams, Platform, Provider, ReviewStatus, Summary, YearStatistics } from "../types"

function fetchAnimes(params: PaginationParams): Promise<PaginatedResult<Anime>> {
    return apiClient.post<PaginatedResult<Anime>>("/api/animes/page", params)
}

function getSummary(): Promise<Summary> {
    return apiClient.get<Summary>("/api/animes/summary")
}

function getYearStatistics(): Promise<YearStatistics> {
    return apiClient.get<YearStatistics>("/api/animes/year-statistics")
}

function reviewAnime(anilist_id: number, platform: Platform, status: ReviewStatus): Promise<void> {
    return apiClient.get<void>(`/api/anime/${anilist_id}/review/${platform}/${status}`)
}

// 任务相关API
function createJob(platform: Platform, year: number, provider: Provider, model: string): Promise<void> {
    return apiClient.get<void>(`/api/job/${platform}/create/${year}/${provider}/${model}`)
}

function runJob(platform: Platform, year: number): Promise<void> {
    return apiClient.get<void>(`/api/job/${platform}/run/${year}`)
}

function pauseJob(platform: Platform, year: number): Promise<void> {
    return apiClient.get<void>(`/api/job/${platform}/pause/${year}`)
}

function resumeJob(platform: Platform, year: number): Promise<void> {
    return apiClient.get<void>(`/api/job/${platform}/resume/${year}`)
}

function removeJob(platform: Platform, year: number): Promise<void> {
    return apiClient.get<void>(`/api/job/${platform}/remove/${year}`)
}

function listJobs(): Promise<JobDetails[]> {
    return apiClient.get<JobDetails[]>('/api/job/list')
}

// 数据导出/导入相关API
function exportAnimes(year: number): Promise<void> {
    return apiClient.get<void>(`/api/export/animes/${year}`)
}

function importAnimes(year: number): Promise<void> {
    return apiClient.get<void>(`/api/import/animes/${year}`)
}

function compactAnimes(): Promise<void> {
    return apiClient.get<void>('/api/compact/animes/dir')
}

export { fetchAnimes, getSummary, getYearStatistics, reviewAnime, createJob, runJob, pauseJob, resumeJob, removeJob, listJobs, exportAnimes, importAnimes, compactAnimes }
