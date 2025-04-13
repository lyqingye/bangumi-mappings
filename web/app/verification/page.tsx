"use client"

import { useState, useEffect, useCallback, useMemo } from "react"
import { motion, AnimatePresence } from "framer-motion"
import { RefreshCw, Search, Check, X, Trash2, AlertTriangle } from "lucide-react"

// 组件导入
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { PageTransition } from "@/components/page-transition"
import { useError } from "@/components/providers/error-provider"
import { useToast } from "@/components/ui/use-toast"
import { AniListPanel } from "@/components/anime-source-panels/anilist-panel"
import { TMDBPanel } from "@/components/anime-source-panels/tmdb-panel"
import { BGMTVPanel } from "@/components/anime-source-panels/bgmtv-panel"
import { Pagination } from "@/components/pagination"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"

// 钩子和工具导入
import { useAniListAnimeDetail, useTMDBAnimeComplete, useBGMTVAnimeDetail } from "@/lib/api/hooks"
import { fetchAnimes, reviewAnime } from "@/lib/api/animes"
import { containerVariants, itemVariants } from "@/animations/variants"
import { ReviewStatus, Platform, Anime, PaginatedResult, Mapping, PaginationParams } from "@/lib/types"
import { getStatusLabel } from "@/lib/utils"

// 过滤标签
const filterTabs = [
  { id: "all", label: "所有", status: null },
  { id: "ready", label: "待审核", status: ReviewStatus.Ready },
  { id: "accepted", label: "已接受", status: ReviewStatus.Accepted },
  { id: "rejected", label: "已拒绝", status: ReviewStatus.Rejected },
  { id: "dropped", label: "已丢弃", status: ReviewStatus.Dropped },
  { id: "unmatched", label: "未匹配", status: ReviewStatus.UnMatched },
]

// 平台名称映射
const PlatformLabels: Record<Platform, string> = {
  [Platform.Tmdb]: "TMDB",
  [Platform.BgmTv]: "Bangumi",
};

// 平台颜色映射
const PlatformColors: Record<Platform, string> = {
  [Platform.Tmdb]: "text-blue-400",
  [Platform.BgmTv]: "text-green-400",
};

/**
 * 审核系统数据管理Hook
 * 负责管理所有数据和状态，与UI分离
 */
function useAnimeVerificationSystem() {
  const { setError } = useError();
  const { toast } = useToast();
  
  // 数据状态
  const [animeList, setAnimeList] = useState<PaginatedResult<Anime> | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  
  // 查询参数
  const [queryParams, setQueryParams] = useState<PaginationParams>({
    page: 1,
    page_size: 10,
    status: ReviewStatus.Ready,
    year: 2025
  });
  
  // 选中项状态
  const [currentAnimeId, setCurrentAnimeId] = useState<number | null>(null);
  const [currentPlatform, setCurrentPlatform] = useState<Platform>(Platform.Tmdb);
  
  // 审核状态
  const [updatingStatus, setUpdatingStatus] = useState<ReviewStatus | null>(null);
  
  // 获取动漫列表
  const fetchAnimeList = useCallback(async (params: PaginationParams) => {
    setIsLoading(true);
    
    try {
      const data = await fetchAnimes(params);
      setAnimeList(data);
      return data;
    } catch (error) {
      console.error("获取动漫列表失败:", error);
      setError("获取动漫列表失败，请稍后重试");
      return null;
    } finally {
      setIsLoading(false);
    }
  }, [setError]);
  
  // 初始化加载
  useEffect(() => {
    fetchAnimeList(queryParams);
  }, []);
  
  // 计算当前选中的动漫数据
  const currentAnime = useMemo(() => 
    currentAnimeId ? animeList?.data.find(a => a.anilist_id === currentAnimeId) : null, 
    [animeList, currentAnimeId]
  );
  
  // 提取并组织映射信息
  const mapping = useMemo(() => {
    if (!currentAnime) return null;
    
    const platformIds = currentAnime.mappings.reduce((acc, mapping) => {
      acc[mapping.platform] = mapping.id;
      return acc;
    }, {} as Record<Platform, string | null>);
    
    return {
      anilist_id: currentAnime.anilist_id,
      mappings: currentAnime.mappings,
      platformIds
    };
  }, [currentAnime]);
  
  // 计算当前选中平台的映射
  const currentMapping = useMemo(() => 
    mapping?.mappings.find(m => m.platform === currentPlatform) || null,
    [mapping, currentPlatform]
  );
  
  // 可用平台列表
  const availablePlatforms = useMemo(() => 
    mapping?.mappings.map(m => m.platform) || [],
    [mapping]
  );
  
  // 更新查询参数并获取数据
  const updateQueryAndFetch = useCallback((newParams: Partial<PaginationParams>) => {
    const updatedParams = { ...queryParams, ...newParams };
    setQueryParams(updatedParams);
    return fetchAnimeList(updatedParams);
  }, [queryParams, fetchAnimeList]);
  
  // 切换标签
  const changeTab = useCallback((tabId: string, additionalParams: Partial<PaginationParams> = {}) => {
    const selectedTab = filterTabs.find(tab => tab.id === tabId);
    updateQueryAndFetch({
      page: 1,
      status: selectedTab?.status ?? null,
      ...additionalParams
    });
    setCurrentAnimeId(null);
  }, [updateQueryAndFetch]);
  
  // 切换页码
  const changePage = useCallback((newPage: number) => {
    updateQueryAndFetch({ page: newPage });
    setCurrentAnimeId(null);
  }, [updateQueryAndFetch]);
  
  // 找到第一个待审核平台
  const findFirstPendingPlatform = useCallback((mappings: Mapping[]): Platform | null => {
    const pendingMapping = mappings.find(m => m.review_status === ReviewStatus.Ready);
    return pendingMapping ? pendingMapping.platform : null;
  }, []);
  
  // 导航到下一个待审核动漫
  const navigateToNextAnime = useCallback(() => {
    if (!animeList?.data || animeList.data.length === 0) return;
    
    const currentIndex = animeList.data.findIndex(a => a.anilist_id === currentAnimeId);
    
    if (currentIndex >= 0 && currentIndex < animeList.data.length - 1) {
      // 列表中有下一个动漫
      setCurrentAnimeId(animeList.data[currentIndex + 1].anilist_id);
    } else if (currentIndex === animeList.data.length - 1 && queryParams.page < Math.ceil((animeList.total || 0) / queryParams.page_size)) {
      // 是最后一个，但有下一页
      const nextPage = queryParams.page + 1;
      updateQueryAndFetch({ page: nextPage }).then(newList => {
        if (newList?.data && newList.data.length > 0) {
          setCurrentAnimeId(newList.data[0].anilist_id);
        }
      });
    } else {
      // 没有更多动漫
      toast({
        title: "提示",
        description: "没有更多动漫需要审核",
        duration: 2000,
      });
    }
  }, [animeList, currentAnimeId, queryParams, updateQueryAndFetch, toast]);
  
  // 核心更新函数 - 处理动画映射状态更新的通用逻辑
  const updateAnimeMappings = useCallback(async (
    animeId: number,
    status: ReviewStatus,
    targetPlatforms: 'current' | 'all' | 'unmatched',
    currentPlatform?: Platform
  ) => {
    if (!animeId) return null;
    
    try {
      setUpdatingStatus(status);
      
      // 验证必要参数
      if (targetPlatforms === 'current' && !currentPlatform) {
        throw new Error('当前平台操作需要指定平台');
      }
      
      // 准备乐观更新
      if (!animeList) return null;
      
      // 创建新的列表和数据，保持不可变性
      const newList = { ...animeList };
      const newData = [...newList.data];
      
      // 找到当前动漫
      const animeIndex = newData.findIndex(a => a.anilist_id === animeId);
      if (animeIndex === -1) return null;
      
      const anime = { ...newData[animeIndex] };
      const newMappings = [...anime.mappings];
      
      // 记录所有需要更新的映射
      const mappingsToUpdate: { platform: Platform, oldStatus: ReviewStatus }[] = [];
      
      // 根据目标平台选择需要更新的映射
      if (targetPlatforms === 'current' && currentPlatform) {
        // 只更新当前平台
        const currentMappingIndex = newMappings.findIndex(m => m.platform === currentPlatform);
        if (currentMappingIndex >= 0 && newMappings[currentMappingIndex].review_status === ReviewStatus.Ready) {
          mappingsToUpdate.push({
            platform: currentPlatform,
            oldStatus: newMappings[currentMappingIndex].review_status
          });
          
          newMappings[currentMappingIndex] = {
            ...newMappings[currentMappingIndex],
            review_status: status
          };
        }
      } else if (targetPlatforms === 'unmatched') {
        // 更新所有UnMatched状态的映射
        newMappings.forEach((mapping, index) => {
          if (mapping.review_status === ReviewStatus.UnMatched) {
            mappingsToUpdate.push({
              platform: mapping.platform,
              oldStatus: mapping.review_status
            });
            
            newMappings[index] = {
              ...mapping,
              review_status: status
            };
          }
        });
        
        // 如果有指定当前平台，确保它也被更新
        if (currentPlatform) {
          const currentMappingIndex = newMappings.findIndex(m => m.platform === currentPlatform);
          if (currentMappingIndex >= 0 && !mappingsToUpdate.some(m => m.platform === currentPlatform)) {
            mappingsToUpdate.push({
              platform: currentPlatform,
              oldStatus: newMappings[currentMappingIndex].review_status
            });
            
            newMappings[currentMappingIndex] = {
              ...newMappings[currentMappingIndex],
              review_status: status
            };
          }
        }
      } else if (targetPlatforms === 'all') {
        // 更新所有Ready状态的映射
        newMappings.forEach((mapping, index) => {
          if (mapping.review_status === ReviewStatus.Ready) {
            mappingsToUpdate.push({
              platform: mapping.platform,
              oldStatus: mapping.review_status
            });
            
            newMappings[index] = {
              ...mapping,
              review_status: status
            };
          }
        });
      }
      
      // 如果没有找到可以更新的映射，提示并返回
      if (mappingsToUpdate.length === 0) {
        toast({
          title: "提示",
          description: "没有可更新的映射",
          duration: 2000,
        });
        return null;
      }
      
      // 更新动漫的映射
      anime.mappings = newMappings;
      newData[animeIndex] = anime;
      
      // 更新列表数据
      newList.data = newData;
      setAnimeList(newList);
      
      // 对所有更新的映射调用API
      const apiCalls = mappingsToUpdate.map(item => 
        reviewAnime(animeId, item.platform, status)
      );
      
      // 并行执行所有API调用
      await Promise.all(apiCalls);
      
      // 构建并显示成功提示
      let toastMessage = '';
      
      if (targetPlatforms === 'current' && currentPlatform) {
        toastMessage = `${PlatformLabels[currentPlatform]} 验证状态已更新为 ${getStatusLabel(status).label}`;
      } else if (targetPlatforms === 'unmatched') {
        toastMessage = `已将${mappingsToUpdate.length}个映射标记为已丢弃`;
      } else {
        const actionText = status === ReviewStatus.Accepted ? "接受" : "拒绝";
        toastMessage = `已将所有${mappingsToUpdate.length}个映射标记为已${actionText}`;
      }
      
      toast({
        title: "状态已更新",
        description: toastMessage,
        duration: 3000,
      });
      
      // 返回更新后的映射以便后续使用
      return {
        updatedMappings: newMappings
      };
      
    } catch (error) {
      console.error("更新状态失败:", error);
      setError("更新状态失败，请稍后重试");
      
      // 发生错误，重新获取数据恢复状态
      fetchAnimeList(queryParams);
      return null;
    } finally {
      setUpdatingStatus(null);
    }
  }, [animeList, setUpdatingStatus, toast, setError, fetchAnimeList, queryParams]);

  // 审核单个平台操作 - 简化版
  const reviewAnimeStatus = useCallback(async (status: ReviewStatus) => {
    if (!currentAnimeId || !currentPlatform || !currentMapping) return;
    
    try {
      // 使用核心函数处理特定状态
      let result;
      
      // 如果是丢弃操作，设置为处理所有未匹配的映射
      if (status === ReviewStatus.Dropped) {
        result = await updateAnimeMappings(currentAnimeId, status, 'unmatched', currentPlatform);
      } else {
        // 否则只处理当前平台
        result = await updateAnimeMappings(currentAnimeId, status, 'current', currentPlatform);
      }
      
      if (!result) return;
      
      // 检查是否所有平台都已审核完毕
      const allVerified = result.updatedMappings.every(m => m.review_status !== ReviewStatus.Ready);
      
      if (allVerified) {
        // 所有平台都已审核，跳转到下一个动漫
        navigateToNextAnime();
      } else {
        // 寻找下一个待审核平台
        const nextPlatform = findFirstPendingPlatform(result.updatedMappings);
        if (nextPlatform) {
          setCurrentPlatform(nextPlatform);
          toast({
            title: "自动切换数据源",
            description: `已切换到 ${PlatformLabels[nextPlatform]} 数据源进行验证`,
            duration: 2000,
          });
        } else {
          navigateToNextAnime();
        }
      }
    } catch (error) {
      console.error("审核操作失败:", error);
    }
  }, [
    currentAnimeId, 
    currentPlatform, 
    currentMapping,
    updateAnimeMappings, 
    navigateToNextAnime, 
    findFirstPendingPlatform, 
    setCurrentPlatform,
    toast
  ]);
  
  // 处理所有平台的状态更新（全部接受或全部拒绝）- 简化版
  const reviewAllPlatforms = useCallback(async (status: ReviewStatus) => {
    if (!currentAnimeId || !mapping) return;
    
    try {
      // 使用核心函数处理所有平台
      const result = await updateAnimeMappings(currentAnimeId, status, 'all');
      
      if (result) {
        // 所有平台都已审核，跳转到下一个动漫
        navigateToNextAnime();
      }
    } catch (error) {
      console.error("批量审核操作失败:", error);
    }
  }, [
    currentAnimeId,
    mapping, 
    updateAnimeMappings,
    navigateToNextAnime
  ]);
  
  // 手动刷新数据
  const refreshList = useCallback(() => {
    return fetchAnimeList(queryParams);
  }, [fetchAnimeList, queryParams]);
  
  // 初始化选择第一个动漫
  useEffect(() => {
    if (animeList?.data && animeList.data.length > 0 && !currentAnimeId) {
      setCurrentAnimeId(animeList.data[0].anilist_id);
    }
  }, [animeList, currentAnimeId]);
  
  // 确保选择了有效的平台
  useEffect(() => {
    if (availablePlatforms.length > 0 && !availablePlatforms.includes(currentPlatform)) {
      setCurrentPlatform(availablePlatforms[0]);
    }
  }, [availablePlatforms, currentPlatform]);
  
  return {
    // 查询和列表状态
    queryParams,
    animeList,
    isLoading,
    
    // 选中和映射状态
    currentAnimeId,
    currentPlatform,
    currentAnime,
    mapping,
    currentMapping,
    availablePlatforms,
    updatingStatus,
    
    // 操作方法
    selectAnime: setCurrentAnimeId,
    selectPlatform: setCurrentPlatform,
    changeTab,
    changePage,
    reviewAnimeStatus,
    reviewAllPlatforms,
    refreshList
  };
}

export default function VerificationPage() {
  const {
    queryParams,
    animeList,
    isLoading,
    
    currentAnimeId,
    currentPlatform,
    mapping,
    currentMapping,
    availablePlatforms,
    updatingStatus,
    
    selectAnime,
    selectPlatform,
    changeTab,
    changePage,
    reviewAnimeStatus,
    reviewAllPlatforms,
    refreshList
  } = useAnimeVerificationSystem();
  
  // 激活的标签
  const [activeTab, setActiveTab] = useState("ready");
  
  // 获取AniList动漫详情
  const { 
    data: anilistData, 
    isLoading: isAnilistLoading, 
    isError: isAnilistError 
  } = useAniListAnimeDetail(currentAnimeId || undefined);
  
  // 获取TMDB详情
  const { 
    data: tmdbData, 
    isLoading: isTmdbLoading, 
    isError: isTmdbError 
  } = useTMDBAnimeComplete(
    mapping?.platformIds?.[Platform.Tmdb]
      ? mapping.platformIds[Platform.Tmdb] 
      : undefined
  );
  
  // 获取BGMTV详情
  const { 
    data: bgmtvData, 
    isLoading: isBgmtvLoading, 
    isError: isBgmtvError 
  } = useBGMTVAnimeDetail(
    mapping?.platformIds?.[Platform.BgmTv]
      ? mapping.platformIds[Platform.BgmTv] 
      : undefined
  );
  
  // 处理标签切换
  const handleTabChange = useCallback((tabId: string) => {
    setActiveTab(tabId);
    changeTab(tabId);
  }, [changeTab]);

  // 处理年份选择
  const handleYearChange = useCallback((value: string) => {
    changeTab(activeTab, {
      year: value === "all" ? null : Number.parseInt(value)
    });
  }, [activeTab, changeTab]);

  // 处理每页显示数量变更
  const handlePageSizeChange = useCallback((value: string) => {
    changeTab(activeTab, {
      page_size: Number.parseInt(value)
    });
  }, [activeTab, changeTab]);
  
  // 渲染平台面板
  const renderPlatformPanel = useCallback((platform: Platform) => {
    const hasData = !!mapping?.platformIds?.[platform];
    
    switch (platform) {
      case Platform.Tmdb:
        return <TMDBPanel detail={tmdbData} mapping={mapping} /> 
      case Platform.BgmTv:
        return <BGMTVPanel 
          data={bgmtvData} 
          mapping={mapping} 
          anilistId={currentAnimeId}
          onStatusUpdated={refreshList}
        /> 
      default:
        return renderNoDataPanel(platform);
    }
  }, [mapping, tmdbData, bgmtvData, currentAnimeId, refreshList]);
  
  // 无数据面板
  const renderNoDataPanel = useCallback((platform: Platform) => (
    <div className="bg-[#111] border border-[#222] rounded-lg p-6 h-[450px] flex items-center justify-center">
      <div className="text-center">
        <AlertTriangle className="h-10 w-10 text-yellow-500 mx-auto mb-4" />
        <h3 className="text-lg font-medium mb-2">无{PlatformLabels[platform]}数据</h3>
        <p className="text-[#777]">无法加载{PlatformLabels[platform]}数据源的数据。</p>
      </div>
    </div>
  ), []);
  
  // 计算详情加载状态
  const isDetailLoading = isLoading || isAnilistLoading || isTmdbLoading || isBgmtvLoading;
  
  return (
    <PageTransition>
      <div className="min-h-screen bg-[#0a0a0a]">
        {/* Header */}
        <motion.div
          className="border-b border-[#222] bg-[#111] w-full"
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
        >
          <div className="w-full px-6 py-3">
            <div className="flex items-center justify-between">
              <h1 className="text-xl font-bold">动漫审核系统</h1>

              <div className="relative w-64">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[#777]" />
                <Input
                  placeholder="搜索动漫..."
                  className="pl-10 bg-[#222] border-[#333] text-white rounded-full"
                />
              </div>
            </div>
          </div>
        </motion.div>

        <div className="w-full px-6 py-6">
          {/* Filter Tabs */}
          <motion.div
            className="bg-[#111] border border-[#222] rounded-lg p-1 mb-6 flex flex-wrap justify-center"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4, delay: 0.3 }}
          >
            {filterTabs.map((tab, index) => (
              <motion.button
                key={tab.id}
                className={`px-6 py-2 rounded-md text-sm ${
                  activeTab === tab.id.toLowerCase()
                    ? "bg-gradient-to-r from-purple-600 to-blue-600 text-white"
                    : "text-[#777] hover:bg-[#222] hover:text-white"
                }`}
                onClick={() => handleTabChange(tab.id.toLowerCase())}
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.3, delay: 0.3 + index * 0.05 }}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
              >
                {tab.label}
              </motion.button>
            ))}
          </motion.div>

          {/* 筛选和刷新行 */}
          <div className="flex items-center justify-between mb-6">
            <div className="flex gap-4">
              {/* 年份筛选 */}
              <motion.div
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.3, delay: 0.4 }}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
              >
                <Select
                  value={queryParams.year?.toString() || "all"}
                  onValueChange={(value) => {
                    handleYearChange(value);
                  }}
                >
                  <SelectTrigger className="w-[130px] bg-[#111] border-[#222] text-[#ccc]">
                    <SelectValue placeholder="年份" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">所有年份</SelectItem>
                    {[...Array(36)].map((_, i) => (
                      <SelectItem key={2025 - i} value={(2025 - i).toString()}>
                        {2025 - i}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </motion.div>

              {/* 每页显示数量 */}
              <motion.div
                initial={{ opacity: 0, x: 10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.3, delay: 0.45 }}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
              >
                <Select 
                  value={queryParams.page_size.toString()} 
                  onValueChange={(value) => {
                    handlePageSizeChange(value);
                  }}
                >
                  <SelectTrigger className="w-[130px] bg-[#111] border-[#222] text-[#ccc]">
                    <SelectValue placeholder="每页显示" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="10">10 条/页</SelectItem>
                    <SelectItem value="20">20 条/页</SelectItem>
                    <SelectItem value="50">50 条/页</SelectItem>
                  </SelectContent>
                </Select>
              </motion.div>
            </div>
            
            {/* 刷新按钮 */}
            <Button
              variant="outline"
              size="sm"
              onClick={refreshList}
              disabled={isLoading}
              className="bg-[#222] border-[#333] text-white hover:bg-[#333]"
            >
              <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? "animate-spin" : ""}`} />
              刷新
            </Button>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-6 gap-8">
            {/* 左侧列表 */}
            <div className="lg:col-span-1">
              <AnimatePresence mode="wait">
                {isLoading ? (
                  <motion.div
                    key="loading"
                    className="space-y-4"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                  >
                    {Array.from({ length: queryParams.page_size }).map((_, i) => (
                      <div 
                        key={`skeleton-${i}`} 
                        className="bg-[#111] border border-[#222] rounded-lg p-3 flex items-center gap-4"
                      >
                        <div className="flex-1">
                          <div className="h-4 w-3/4 bg-[#222] rounded mb-2 animate-pulse"></div>
                          <div className="h-3 w-1/2 bg-[#222] rounded animate-pulse"></div>
                        </div>
                      </div>
                    ))}
                  </motion.div>
                ) : (
                  <motion.div
                    key="content"
                    className="space-y-4"
                    variants={containerVariants}
                    initial="hidden"
                    animate="visible"
                    exit={{ opacity: 0 }}
                  >
                    {animeList?.data && animeList.data.length > 0 ? (
                      animeList.data.map((anime: Anime, index: number) => (
                        <motion.div 
                          key={anime.anilist_id}
                          variants={itemVariants}
                          custom={index}
                          whileHover={{ scale: 1.02 }}
                          className={`bg-[#111] border ${
                            currentAnimeId === anime.anilist_id 
                              ? 'border-purple-600' 
                              : 'border-[#222]'
                          } rounded-lg p-3 flex items-center gap-4 cursor-pointer`}
                          onClick={() => selectAnime(anime.anilist_id)}
                        >
                          <div className="flex-1">
                            <div className="font-medium text-sm mb-1 line-clamp-1">
                              {anime.titles?.[0] || (anilistData && currentAnimeId === anime.anilist_id 
                                ? (anilistData.title?.romaji || anilistData.title?.english || `ID:${anime.anilist_id}`)
                                : `ID:${anime.anilist_id}`
                              )}
                            </div>
                            <div className="flex flex-wrap gap-2 text-xs">
                              {anime.mappings.map((mapping: Mapping) => (
                                <div key={mapping.platform} className="flex items-center">
                                  <span className="text-[#777]">{PlatformLabels[mapping.platform]}:</span>
                                  <span className={
                                    mapping.review_status === ReviewStatus.Ready 
                                      ? "text-yellow-500 ml-1" 
                                      : mapping.review_status === ReviewStatus.Accepted 
                                      ? "text-green-500 ml-1" 
                                      : "text-red-500 ml-1"
                                  }>
                                    {getStatusLabel(mapping.review_status).label}
                                  </span>
                                </div>
                              ))}
                            </div>
                          </div>
                        </motion.div>
                      ))
                    ) : (
                      <motion.div
                        className="bg-[#111] border border-[#222] rounded-lg p-8 text-center"
                        variants={itemVariants}
                      >
                        <p className="text-[#777]">没有找到匹配的动漫。请尝试调整筛选条件。</p>
                      </motion.div>
                    )}
                  </motion.div>
                )}
              </AnimatePresence>

              {/* 分页控件 */}
              {animeList && animeList.total > 0 && (
                <Pagination
                  currentPage={queryParams.page}
                  totalItems={animeList.total}
                  pageSize={animeList.page_size}
                  onPageChange={changePage}
                  disabled={isLoading}
                />
              )}
            </div>

            {/* 右侧详情 */}
            <div className="lg:col-span-5">
              {isDetailLoading ? (
                <div className="animate-pulse space-y-6">
                  <div className="h-8 w-1/3 rounded bg-[#222]"></div>
                  <div className="h-4 w-1/4 rounded bg-[#222]"></div>
                  <div className="h-64 rounded bg-[#222]"></div>
                  <div className="h-32 rounded bg-[#222]"></div>
                </div>
              ) : currentAnimeId && mapping ? (
                <div className="flex flex-col h-full">
                  {/* 基本信息 */}
                  <div className="flex-grow">
                    {/* 基本信息 */}
                    <div className="mb-6 p-4 bg-[#111] border border-[#222] rounded-lg">
                      <h2 className="text-3xl font-bold">
                        {anilistData?.title?.english || anilistData?.title?.romaji || "Unknown Title"}
                      </h2>
                      <p className="text-[#777] mb-3">{anilistData?.title?.native || ""}</p>

                      <div className="flex flex-wrap gap-2 mt-3">
                        <Badge className="bg-purple-600 text-white">{anilistData?.format || "Unknown"}</Badge>
                        <Badge className="bg-blue-600 text-white">{anilistData?.seasonYear || "Unknown"}</Badge>
                        <Badge className="bg-[#333] text-white">{anilistData?.episodes || 0} 集</Badge>
                      </div>
                    </div>

                    {/* 数据源比较 - 三列布局 */}
                    <div className="mb-6">
                      <h3 className="text-lg font-medium mb-4 text-white">数据源比较</h3>
                      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                        {/* AniList */}
                        <div>
                          <div className="bg-[#111] border border-[#222] rounded-lg overflow-hidden">
                            <div className="bg-purple-900/30 text-purple-300 py-2 px-4 font-medium border-b border-[#222]">
                              AniList
                            </div>
                            <div className="p-4">
                              {anilistData && <AniListPanel data={anilistData} />}
                            </div>
                          </div>
                        </div>

                        {/* TMDB */}
                        <div>
                          <div className="bg-[#111] border border-[#222] rounded-lg overflow-hidden">
                            <div className="bg-blue-900/30 text-blue-300 py-2 px-4 font-medium border-b border-[#222]">
                              TMDB
                            </div>
                            <div className="p-4">
                              {renderPlatformPanel(Platform.Tmdb)}
                            </div>
                          </div>
                        </div>

                        {/* Bangumi */}
                        <div>
                          <div className="bg-[#111] border border-[#222] rounded-lg overflow-hidden">
                            <div className="bg-green-900/30 text-green-300 py-2 px-4 font-medium border-b border-[#222]">
                              Bangumi
                            </div>
                            <div className="p-4">
                              {renderPlatformPanel(Platform.BgmTv)}
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* 审核操作栏 - 移到底部 */}
                  <div className="mt-6 bg-[#111] border border-[#222] rounded-lg p-6 sticky bottom-0">
                    <div className="flex flex-col gap-6">
                      {/* 标题显示当前选择的平台 */}
                      <div className="w-full">
                        <h3 className="text-lg font-medium text-white">当前审核: <span className={PlatformColors[currentPlatform]}>{PlatformLabels[currentPlatform]}</span></h3>
                      </div>
                      
                      {/* 单平台审核按钮组和平台选择器*/}
                      <div className="flex flex-wrap items-center justify-between gap-4">
                        {/* 单平台审核按钮组 */}
                        <div className="flex items-center gap-4">
                          <Button
                            size="sm"
                            onClick={() => reviewAnimeStatus(ReviewStatus.Accepted)}
                            disabled={updatingStatus !== null || !currentMapping}
                            className="bg-gradient-to-r from-[#8a2be2] to-[#4169e1] hover:opacity-90 text-white border-none rounded-md px-6 py-5 shadow-md transition-all"
                          >
                            {updatingStatus === ReviewStatus.Accepted ? (
                              <RefreshCw className="h-4 w-4 animate-spin mr-2" />
                            ) : (
                              <Check className="h-4 w-4 mr-2" />
                            )}
                            接受
                          </Button>

                          <Button
                            size="sm"
                            onClick={() => reviewAnimeStatus(ReviewStatus.Rejected)}
                            disabled={updatingStatus !== null || !currentMapping}
                            className="bg-gradient-to-r from-[#ff1493] to-[#ff8c00] hover:opacity-90 text-white border-none rounded-md px-6 py-5 shadow-md transition-all"
                          >
                            {updatingStatus === ReviewStatus.Rejected ? (
                              <RefreshCw className="h-4 w-4 animate-spin mr-2" />
                            ) : (
                              <X className="h-4 w-4 mr-2" />
                            )}
                            拒绝
                          </Button>

                          <Button
                            size="sm"
                            onClick={() => reviewAnimeStatus(ReviewStatus.Dropped)}
                            disabled={updatingStatus !== null || !currentMapping}
                            className="bg-gradient-to-r from-[#6a82fb] to-[#67d1ff] hover:opacity-90 text-white border-none rounded-md px-6 py-5 shadow-md transition-all"
                          >
                            {updatingStatus === ReviewStatus.Dropped ? (
                              <RefreshCw className="h-4 w-4 animate-spin mr-2" />
                            ) : (
                              <Trash2 className="h-4 w-4 mr-2" />
                            )}
                            丢弃
                          </Button>
                        </div>
                        
                        {/* 平台选择器 */}
                        {availablePlatforms.length > 1 && (
                          <div className="flex items-center gap-2 bg-[#222] rounded-lg px-4 py-2">
                            <Select
                              value={currentPlatform}
                              onValueChange={(value) => selectPlatform(value as Platform)}
                            >
                              <SelectTrigger className="w-[150px] bg-[#111] border-[#222] text-[#ccc]">
                                <SelectValue placeholder="选择数据源" />
                              </SelectTrigger>
                              <SelectContent>
                                {availablePlatforms.map(platform => (
                                  <SelectItem key={platform} value={platform}>
                                    <span className={PlatformColors[platform]}>{PlatformLabels[platform]}</span>
                                  </SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                          </div>
                        )}
                      </div>
                      
                      {/* 横线分隔符 */}
                      <div className="border-t border-[#333] my-2"></div>
                      
                      {/* 所有平台批量操作按钮 */}
                      <div className="w-full">
                        <h3 className="text-sm font-medium text-[#999] mb-4">批量操作所有平台:</h3>
                        <div className="flex items-center gap-4">
                          <Button
                            size="sm"
                            onClick={() => reviewAllPlatforms(ReviewStatus.Accepted)}
                            disabled={updatingStatus !== null || !mapping}
                            className="bg-gradient-to-r from-[#8a2be2] to-[#4169e1] hover:opacity-90 text-white border-none rounded-md px-6 py-5 shadow-md transition-all"
                          >
                            {updatingStatus === ReviewStatus.Accepted ? (
                              <RefreshCw className="h-4 w-4 animate-spin mr-2" />
                            ) : (
                              <Check className="h-4 w-4 mr-2" />
                            )}
                            全部接受
                          </Button>

                          <Button
                            size="sm"
                            onClick={() => reviewAllPlatforms(ReviewStatus.Rejected)}
                            disabled={updatingStatus !== null || !mapping}
                            className="bg-gradient-to-r from-[#ff1493] to-[#ff8c00] hover:opacity-90 text-white border-none rounded-md px-6 py-5 shadow-md transition-all"
                          >
                            {updatingStatus === ReviewStatus.Rejected ? (
                              <RefreshCw className="h-4 w-4 animate-spin mr-2" />
                            ) : (
                              <X className="h-4 w-4 mr-2" />
                            )}
                            全部拒绝
                          </Button>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="bg-[#111] border border-[#222] rounded-lg p-8 text-center">
                  <p className="text-[#777]">请从左侧列表中选择一个动漫进行审核</p>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </PageTransition>
  );
} 