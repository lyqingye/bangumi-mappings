"use client"

import { useQuery, useMutation, useQueryClient, UseQueryResult } from "@tanstack/react-query"
import { getAniListAnimeDetail } from "./anilist"
import { getTMDBAnimeDetail, getTMDBAnimeCredits } from "./tmdb"
import { getBGMTVAnimeDetail } from "./bgmtv"

// 查询键前缀
const QUERY_KEYS = {
  ANILIST: "anilist",
  TMDB: "tmdb",
  BGMTV: "bgmtv",
}

/**
 * 获取 AniList 动漫详情
 */
export function useAniListAnimeDetail(id: string | number | undefined) {
  return useQuery({
    queryKey: [QUERY_KEYS.ANILIST, id],
    queryFn: () => (id ? getAniListAnimeDetail(id) : Promise.reject("No ID provided")),
    enabled: !!id,
  })
}

/**
 * 获取 TMDB 动漫详情
 */
export function useTMDBAnimeDetail(id: string | number | undefined) {
  return useQuery({
    queryKey: [QUERY_KEYS.TMDB, "detail", id],
    queryFn: () => (id ? getTMDBAnimeDetail(id) : Promise.reject("No ID provided")),
    enabled: !!id,
  })
}

/**
 * 获取 TMDB 演员表
 */
export function useTMDBAnimeCredits(id: string | number | undefined) {
  return useQuery({
    queryKey: [QUERY_KEYS.TMDB, "credits", id],
    queryFn: () => (id ? getTMDBAnimeCredits(id) : Promise.reject("No ID provided")),
    enabled: !!id,
  })
}

/**
 * 获取 TMDB 完整数据（详情和演员表）
 */
export function useTMDBAnimeComplete(id: string | number | undefined) {
  const detailQuery = useTMDBAnimeDetail(id)
  const creditsQuery = useTMDBAnimeCredits(id)

  const isLoading = detailQuery.isLoading || creditsQuery.isLoading
  const isError = detailQuery.isError || creditsQuery.isError
  const error = detailQuery.error || creditsQuery.error

  const data =
    detailQuery.data && creditsQuery.data ? { detail: detailQuery.data, credits: creditsQuery.data } : undefined

  return {
    data,
    isLoading,
    isError,
    error,
    detailQuery,
    creditsQuery,
  }
}

/**
 * 获取 BgmTV 动漫详情
 */
export function useBGMTVAnimeDetail(id: string | number | undefined) {
  return useQuery({
    queryKey: [QUERY_KEYS.BGMTV, id],
    queryFn: () => (id ? getBGMTVAnimeDetail(id) : Promise.reject("No ID provided")),
    enabled: !!id,
  })
}