import { apiClient } from "./api-client"
import type { Anime, JobDetails, PaginatedResult, PaginationParams, Platform, Provider, ReviewStatus } from "../types"

function fetchAnimes(params: PaginationParams): Promise<PaginatedResult<Anime>> {
    return apiClient.post<PaginatedResult<Anime>>("/api/animes/page", params)
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

export { fetchAnimes, reviewAnime, createJob, runJob, pauseJob, resumeJob, removeJob, listJobs }
