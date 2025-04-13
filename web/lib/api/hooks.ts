"use client"

import { useQueries, useQuery, useMutation, useQueryClient, UseQueryResult } from "@tanstack/react-query"
import { getAniListAnimeDetail } from "./anilist"
import { getTMDBAnimeDetail, getTMDBEpisodeGroups, getTMDBEpisodeGroupDetail, TMDBSeason, TMDBEpisodeGroupDetail } from "./tmdb"
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
 * 将 Episode Group 转换为 Seasons 结构
 */
function convertEpisodeGroupsToSeasons(episodeGroupDetail: TMDBEpisodeGroupDetail): TMDBSeason[] {
  return episodeGroupDetail.groups.map(group => ({
    id: parseInt(group.id) || 0,
    name: group.name,
    overview: episodeGroupDetail.description,
    season_number: group.order,
    episode_count: group.episodes.length,
    poster_path: null, // Episode groups 通常没有专属海报
    air_date: group.episodes[0]?.air_date || ""
  }))
}

/**
 * 获取 TMDB 完整数据（详情、演员表、以及剧集分组）
 */
export function useTMDBAnimeComplete(id: string | number | undefined) {
  // 获取基础动漫详情
  const detailQuery = useTMDBAnimeDetail(id)

  // 获取剧集分组信息
  const episodeGroupsQuery = useQuery({
    queryKey: [QUERY_KEYS.TMDB, "episodeGroups", id],
    queryFn: () => (id ? getTMDBEpisodeGroups(id) : Promise.reject("No ID provided")),
    enabled: !!id && !detailQuery.isLoading && !detailQuery.isError,
  })

  // 如果有剧集分组，获取第一个分组的详情
  const firstGroupId = episodeGroupsQuery.data?.results[0]?.id
  const episodeGroupDetailQuery = useQuery({
    queryKey: [QUERY_KEYS.TMDB, "episodeGroupDetail", firstGroupId],
    queryFn: () => (firstGroupId ? getTMDBEpisodeGroupDetail(firstGroupId) : Promise.reject("No group ID")),
    enabled: !!firstGroupId,
  })

  const isLoading = detailQuery.isLoading || 
                    episodeGroupsQuery.isLoading || 
                    (!!firstGroupId && episodeGroupDetailQuery.isLoading)
                    
  const isError = detailQuery.isError || 
                 episodeGroupsQuery.isError || 
                 (!!firstGroupId && episodeGroupDetailQuery.isError)
                 
  const error = detailQuery.error || 
               episodeGroupsQuery.error || 
               (firstGroupId ? episodeGroupDetailQuery.error : null)

  // 合并数据，如果有剧集分组则替换seasons
  let mergedData = detailQuery.data
  if (mergedData && episodeGroupDetailQuery.data) {
    // 创建数据的深拷贝，避免修改原始缓存数据
    mergedData = {
      ...mergedData,
      seasons: convertEpisodeGroupsToSeasons(episodeGroupDetailQuery.data)
    }
  }

  return {
    data: mergedData,
    isLoading,
    isError,
    error,
    detailQuery,
    episodeGroupsQuery,
    episodeGroupDetailQuery,
    hasEpisodeGroups: !!firstGroupId
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