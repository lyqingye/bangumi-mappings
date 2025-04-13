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
 * 获取 TMDB 完整数据（详情和演员表）
 */
export function useTMDBAnimeComplete(id: string | number | undefined) {
  const detailQuery = useTMDBAnimeDetail(id)

  const isLoading = detailQuery.isLoading
  const isError = detailQuery.isError
  const error = detailQuery.error

  const data = detailQuery.data

  return {
    data,
    isLoading,
    isError,
    error,
    detailQuery,
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