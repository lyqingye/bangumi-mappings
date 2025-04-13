"use client"

import { useState, useEffect, useCallback } from "react"
import { Search, Loader2, Film, Tv, Calendar, Star, Hash, AlertTriangle } from "lucide-react"
import Image from "next/image"
import Link from "next/link"
import { useToast } from "@/hooks/use-toast"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { 
  searchTMDBAnime, 
  searchTMDBMovie,
  getTMDBAnimeDetail,
  getTMDBEpisodeGroups,
  getTMDBEpisodeGroupDetail,
  getTMDBImageUrl, 
  TMDBSearchResult, 
  TMDBMovieSearchResult,
  TMDBAnimeDetail,
  TMDBSeason,
  TMDBEpisodeGroupDetail
} from "@/lib/api/tmdb"
import { manualMapping } from "@/lib/api/animes"
import { Platform } from "@/lib/types"

interface TMDBSearchDialogProps {
  isOpen: boolean
  setIsOpen: (open: boolean) => void
  anilistId: number | null
  onMappingSuccess?: () => void
}

// 将Episode Group转换为Seasons结构（与hooks.ts中相同的逻辑）
function convertEpisodeGroupsToSeasons(episodeGroupDetail: TMDBEpisodeGroupDetail): TMDBSeason[] {
  return episodeGroupDetail.groups.map(group => ({
    id: parseInt(group.id) || 0,
    name: group.name,
    overview: episodeGroupDetail.description,
    season_number: group.order,
    episode_count: group.episodes.length,
    poster_path: null, // Episode groups通常没有专属海报
    air_date: group.episodes[0]?.air_date || ""
  }))
}

type SearchType = "tv" | "movie"

// 电视剧搜索结果项，包含季节信息
interface TVSearchItem {
  id: number;
  name: string;
  originalName: string;
  posterPath: string | null;
  firstAirDate: string;
  voteAverage: number;
  seasons: TMDBSeason[];
  loading: boolean;
}

export function TMDBSearchDialog({ isOpen, setIsOpen, anilistId, onMappingSuccess }: TMDBSearchDialogProps) {
  const [searchType, setSearchType] = useState<SearchType>("tv")
  const [searchTerm, setSearchTerm] = useState("")
  const [tvItems, setTvItems] = useState<TVSearchItem[]>([])
  const [movieResults, setMovieResults] = useState<TMDBMovieSearchResult[]>([])
  const [isSearching, setIsSearching] = useState(false)
  
  const [selectedTVId, setSelectedTVId] = useState<number | null>(null)
  const [selectedSeasonId, setSelectedSeasonId] = useState<number | null>(null)
  const [selectedMovieId, setSelectedMovieId] = useState<number | null>(null)
  
  const [isSaving, setIsSaving] = useState(false)
  
  const { toast } = useToast()

  // 清空选中的内容，当对话框关闭时
  useEffect(() => {
    if (!isOpen) {
      setSelectedTVId(null)
      setSelectedSeasonId(null)
      setSelectedMovieId(null)
      setTvItems([])
      setMovieResults([])
      setSearchTerm("")
    }
  }, [isOpen])

  // 处理搜索逻辑
  const handleSearch = async () => {
    if (!searchTerm.trim()) return
    
    setIsSearching(true)
    setSelectedTVId(null)
    setSelectedSeasonId(null)
    setSelectedMovieId(null)
    setTvItems([])
    setMovieResults([])
    
    try {
      if (searchType === "tv") {
        const response = await searchTMDBAnime(searchTerm.trim())
        
        // 创建初始TV项目，稍后加载季度信息
        const initialTvItems = response.results.map(tv => ({
          id: tv.id,
          name: tv.name,
          originalName: tv.original_name,
          posterPath: tv.poster_path,
          firstAirDate: tv.first_air_date,
          voteAverage: tv.vote_average,
          seasons: [],
          loading: true
        }))
        
        setTvItems(initialTvItems)
        
        // 为每个电视剧加载季度信息
        const tvItemsWithSeasons = await Promise.all(
          initialTvItems.map(async (item) => {
            try {
              // 获取TV详情
              const detail = await getTMDBAnimeDetail(item.id)
              
              // 尝试获取剧集分组
              try {
                const episodeGroups = await getTMDBEpisodeGroups(item.id)
                if (episodeGroups.results && episodeGroups.results.length > 0) {
                  // 如果有剧集分组，获取第一个分组的详情
                  const groupDetail = await getTMDBEpisodeGroupDetail(episodeGroups.results[0].id)
                  // 转换为Seasons格式
                  const convertedSeasons = convertEpisodeGroupsToSeasons(groupDetail)
                  return {
                    ...item,
                    seasons: convertedSeasons,
                    loading: false
                  }
                } else {
                  // 如果没有剧集分组，使用普通seasons
                  return {
                    ...item,
                    seasons: detail.seasons || [],
                    loading: false
                  }
                }
              } catch (error) {
                // 如果获取分组失败，使用普通seasons
                return {
                  ...item,
                  seasons: detail.seasons || [],
                  loading: false
                }
              }
            } catch (error) {
              return {
                ...item,
                loading: false
              }
            }
          })
        )
        
        setTvItems(tvItemsWithSeasons)
      } else {
        const response = await searchTMDBMovie(searchTerm.trim())
        setMovieResults(response.results)
      }
    } catch (error) {
      toast({
        title: "搜索失败",
        description: "无法获取TMDB搜索结果，请稍后重试",
        variant: "destructive"
      })
    } finally {
      setIsSearching(false)
    }
  }

  // 输入框按下Enter键处理
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && searchTerm.trim()) {
      handleSearch()
    }
  }

  // 选择电视剧项目
  const handleSelectTV = (tvId: number) => {
    setSelectedTVId(tvId)
    setSelectedSeasonId(null)
    setSelectedMovieId(null)
  }

  // 选择电视剧季节
  const handleSelectSeason = useCallback(
    (tvId: number, seasonId: number) => {
      console.log(`选择了TV: ${tvId}, 季节: ${seasonId}`)
      setSelectedMovieId(null)
      setSelectedTVId(tvId)
      setSelectedSeasonId(seasonId)
      
      // 等待状态更新后检查
      setTimeout(() => {
        console.log("更新后的状态 - selectedTVId:", tvId, "selectedSeasonId:", seasonId)
      }, 0)
    },
    []
  )

  // 选择电影
  const handleSelectMovie = useCallback((movieId: number) => {
    console.log(`选择了电影: ${movieId}`)
    setSelectedTVId(null)
    setSelectedSeasonId(null)
    setSelectedMovieId(movieId)
    
    // 等待状态更新后检查
    setTimeout(() => {
      console.log("更新后的状态 - selectedMovieId:", movieId)
    }, 0)
  }, [])

  // 保存映射
  const handleSaveMapping = async () => {
    // 检查是否有有效的AniList ID
    if (!anilistId) {
      toast({
        title: "操作无法完成",
        description: "无法获取AniList ID，请在番剧详情页进行操作",
        variant: "destructive"
      })
      return
    }

    setIsSaving(true)

    try {
      if (selectedTVId) {
        console.log("保存TV映射, selectedTVId:", selectedTVId, "selectedSeasonId:", selectedSeasonId)
        
        // 查找选中的TV节目
        const selectedTV = tvItems.find(tv => tv.id === selectedTVId)
        console.log("找到的TV节目:", selectedTV)
        
        if (selectedTV) {
          // 查找选中的季节
          const selectedSeason = selectedTV.seasons.find(
            season => Number(season.id) === Number(selectedSeasonId)
          )
          console.log("找到的季节:", selectedSeason)
          
          if (selectedSeason) {
            await manualMapping(
              anilistId,
              Platform.Tmdb,
              selectedTVId.toString(),
              selectedSeason.season_number
            )
            
            if (onMappingSuccess) {
              onMappingSuccess()
            }
            
            console.log("TV映射保存成功!")
            setIsOpen(false)
          } else {
            console.error("未找到选中的季节!")
          }
        } else {
          console.error("未找到选中的TV节目!")
        }
      } else if (selectedMovieId) {
        console.log("保存电影映射, selectedMovieId:", selectedMovieId)
        
        await manualMapping(
          anilistId,
          Platform.Tmdb,
          selectedMovieId.toString(),
          null // 电影没有季度
        )
        
        if (onMappingSuccess) {
          onMappingSuccess()
        }
        
        console.log("电影映射保存成功!")
        setIsOpen(false)
      }
    } catch (error) {
      console.error("保存映射失败:", error)
      toast({
        title: "保存映射失败",
        description: "请稍后重试",
        variant: "destructive"
      })
    } finally {
      setIsSaving(false)
    }
  }

  // 获取当前选中的项目名称
  const getSelectedName = () => {
    if (selectedMovieId) {
      const movie = movieResults?.find(m => m.id === selectedMovieId);
      return movie?.title || movie?.original_title || "未知电影";
    } else if (selectedTVId && selectedSeasonId) {
      const tv = tvItems?.find(t => t.id === selectedTVId);
      const season = tv?.seasons?.find(s => s.id === selectedSeasonId);
      return `${tv?.name || tv?.originalName || "未知剧集"} - ${season?.name || `第${season?.season_number}季`}`;
    }
    return "";
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogContent className="sm:max-w-[700px] bg-black border border-[#2a2a40] text-white max-h-[90vh] overflow-hidden flex flex-col shadow-xl rounded-xl">
        <DialogHeader className="border-b border-[#2a2a40] pb-4 px-6 pt-6">
          <DialogTitle className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-purple-400 to-blue-500">TMDB 搜索 {anilistId ? `(ID: ${anilistId})` : ""}</DialogTitle>
        </DialogHeader>
        
        <div className="flex items-center gap-3 mt-6 mb-5 px-6">
          <Tabs defaultValue="tv" className="w-[180px]" onValueChange={(v) => setSearchType(v as SearchType)}>
            <TabsList className="bg-[#1e1e30] border border-[#2a2a40] p-1">
              <TabsTrigger value="tv" className="text-sm data-[state=active]:bg-gradient-to-r data-[state=active]:from-[#8a2be2] data-[state=active]:to-[#4169e1] data-[state=active]:text-white data-[state=inactive]:text-[#8a8aaa]">
                <Tv className="h-3.5 w-3.5 mr-1.5" />电视剧
              </TabsTrigger>
              <TabsTrigger value="movie" className="text-sm data-[state=active]:bg-gradient-to-r data-[state=active]:from-[#8a2be2] data-[state=active]:to-[#4169e1] data-[state=active]:text-white data-[state=inactive]:text-[#8a8aaa]">
                <Film className="h-3.5 w-3.5 mr-1.5" />电影
              </TabsTrigger>
            </TabsList>
          </Tabs>
          
          <div className="flex-1 flex gap-2">
            <div className="relative flex-1">
              <Input 
                placeholder="输入名称搜索..." 
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="bg-[#1e1e30] border-[#2a2a40] text-white pr-10 focus-visible:ring-purple-500 transition-all duration-200 h-10"
                onKeyDown={handleKeyDown}
              />
              <Search className="h-4 w-4 absolute right-3 top-3 text-[#8a8aaa]" />
            </div>
            <Button 
              onClick={handleSearch}
              disabled={isSearching || !searchTerm.trim()}
              className="bg-gradient-to-r from-[#8a2be2] to-[#4169e1] hover:opacity-90 text-white shadow-md transition-all duration-200 h-10"
            >
              {isSearching ? <Loader2 className="h-4 w-4 animate-spin" /> : "搜索"}
            </Button>
          </div>
        </div>
        
        <Tabs defaultValue="tv" className="flex-1 px-6" value={searchType}>
          <TabsContent value="tv" className="m-0 p-0 h-[450px] overflow-y-auto pr-2 space-y-5">
            {isSearching ? (
              <div className="h-full flex items-center justify-center">
                <div className="flex flex-col items-center">
                  <Loader2 className="h-10 w-10 animate-spin text-purple-500 mb-3" />
                  <p className="text-[#8a8aaa] text-sm">搜索中...</p>
                </div>
              </div>
            ) : tvItems.length === 0 ? (
              <div className="h-full flex items-center justify-center">
                <div className="flex flex-col items-center text-center px-6">
                  <Search className="h-12 w-12 text-[#2a2a40] mb-4" />
                  <p className="text-[#8a8aaa] text-sm max-w-xs">
                    {searchTerm.trim() ? "未找到相关电视剧，请尝试其他关键词" : "请输入名称开始搜索"}
                  </p>
                </div>
              </div>
            ) : (
              <div className="space-y-6">
                {tvItems.map((item) => (
                  <div key={item.id} className="rounded-xl overflow-hidden bg-[#121218] border border-[#2a2a40] transition-all duration-200 hover:shadow-md hover:shadow-purple-900/20">
                    <div 
                      className={`p-4 cursor-pointer transition-all ${
                        selectedTVId === item.id && !selectedSeasonId
                          ? "bg-gradient-to-r from-[#20203a] to-[#1a1a2e] border-l-4 border-l-purple-500" 
                          : "border-l-4 border-transparent"
                      }`}
                      onClick={() => handleSelectTV(item.id)}
                    >
                      <div className="flex gap-4">
                        <div className="w-16 h-24 relative flex-shrink-0 rounded-md overflow-hidden bg-[#0e0e18] shadow-md">
                          <Image 
                            src={item.posterPath ? getTMDBImageUrl(item.posterPath, "w92") : "/placeholder.svg"} 
                            alt={item.name}
                            fill
                            className="object-cover"
                          />
                        </div>
                        
                        <div className="flex-1">
                          <h3 className="font-semibold text-base text-white tracking-wide">
                            {item.name}
                          </h3>
                          <p className="text-sm text-[#8a8aaa] mt-0.5">{item.originalName}</p>
                          
                          <div className="flex items-center gap-2 mt-3">
                            <Badge variant="outline" className="text-xs bg-[#2a2a40]/50 border-[#3d3d5c] px-2 py-0.5 rounded-md">
                              <Tv className="h-3 w-3 mr-1" /> 电视剧
                            </Badge>
                            {item.firstAirDate && (
                              <Badge variant="outline" className="text-xs bg-[#2a2a40]/50 border-[#3d3d5c] px-2 py-0.5 rounded-md">
                                <Calendar className="h-3 w-3 mr-1" /> {item.firstAirDate.substring(0, 4)}
                              </Badge>
                            )}
                            {item.voteAverage > 0 && (
                              <Badge variant="outline" className="text-xs bg-[#2a2a40]/50 border-[#3d3d5c] px-2 py-0.5 rounded-md">
                                <Star className="h-3 w-3 mr-1 text-amber-400" /> {item.voteAverage.toFixed(1)}
                              </Badge>
                            )}
                            <Link
                              href={`https://www.themoviedb.org/tv/${item.id}`}
                              target="_blank"
                              className="text-xs text-purple-400 hover:text-purple-300 ml-auto transition-colors"
                              onClick={(e) => e.stopPropagation()}
                            >
                              查看详情
                            </Link>
                          </div>
                        </div>
                      </div>
                    </div>
                    
                    {/* 季度信息，直接显示，无需点击 */}
                    <div className="p-4 bg-[#0a0a10]">
                      {item.loading ? (
                        <div className="py-4 flex justify-center">
                          <Loader2 className="h-5 w-5 animate-spin text-purple-500" />
                        </div>
                      ) : item.seasons.length === 0 ? (
                        <div className="py-3 text-center text-sm text-[#8a8aaa]">
                          该剧集没有季度信息
                        </div>
                      ) : (
                        <>
                          <p className="text-xs text-[#8a8aaa] mb-3 font-medium uppercase tracking-wider">季度列表</p>
                          <div className="grid grid-cols-2 gap-3">
                            {item.seasons.map((season) => (
                              <div 
                                key={`season-${item.id}-${season.id}`}
                                className={`p-3 rounded-lg cursor-pointer transition-all duration-200 border ${
                                  selectedTVId === item.id && Number(selectedSeasonId) === Number(season.id)
                                    ? "border-purple-500 bg-purple-900/20 shadow-sm shadow-purple-900/40" 
                                    : "border-[#2a2a40] hover:border-purple-500/50 hover:bg-[#20203a]"
                                }`}
                                onClick={() => {
                                  console.log(`点击季度 - item.id: ${item.id}, season.id: ${season.id}, 类型: ${typeof season.id}`);
                                  handleSelectSeason(item.id, season.id);
                                }}
                              >
                                <div className="flex-1">
                                  <p className="text-sm font-medium text-white">
                                    {season.name}
                                  </p>
                                  <div className="flex flex-wrap items-center gap-1 mt-2">
                                    <span className="text-xs text-[#8a8aaa]">
                                      <span className="inline-block px-1.5 py-0.5 bg-[#2a2a40]/40 rounded mr-1">S{season.season_number}</span>
                                    </span>
                                    <span className="text-xs text-[#8a8aaa]">
                                      <span className="inline-block px-1.5 py-0.5 bg-[#2a2a40]/40 rounded mr-1">{season.episode_count}集</span>
                                    </span>
                                    {season.air_date && (
                                      <span className="text-xs text-[#8a8aaa]">
                                        <span className="inline-block px-1.5 py-0.5 bg-[#2a2a40]/40 rounded">{season.air_date.substring(0, 4)}</span>
                                      </span>
                                    )}
                                  </div>
                                </div>
                              </div>
                            ))}
                          </div>
                        </>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </TabsContent>
          
          <TabsContent value="movie" className="m-0 p-0 h-[450px] overflow-y-auto pr-2">
            {isSearching ? (
              <div className="h-full flex items-center justify-center">
                <div className="flex flex-col items-center">
                  <Loader2 className="h-10 w-10 animate-spin text-purple-500 mb-3" />
                  <p className="text-[#8a8aaa] text-sm">搜索中...</p>
                </div>
              </div>
            ) : movieResults.length === 0 ? (
              <div className="h-full flex items-center justify-center">
                <div className="flex flex-col items-center text-center px-6">
                  <Search className="h-12 w-12 text-[#2a2a40] mb-4" />
                  <p className="text-[#8a8aaa] text-sm max-w-xs">
                    {searchTerm.trim() ? "未找到相关电影，请尝试其他关键词" : "请输入名称开始搜索"}
                  </p>
                </div>
              </div>
            ) : (
              <div className="space-y-4">
                {movieResults.map((movie) => (
                  <div 
                    key={movie.id}
                    className={`p-4 rounded-xl cursor-pointer transition-all duration-200 flex gap-4 ${
                      selectedMovieId === movie.id 
                        ? "bg-[#121218] border-l-4 border-l-purple-500 border-t border-r border-b border-[#2a2a40]" 
                        : "border border-[#2a2a40] bg-[#121218] hover:border-purple-500/50 hover:bg-[#0c0c14]"
                    }`}
                    onClick={() => handleSelectMovie(movie.id)}
                  >
                    <div className="w-16 h-24 relative flex-shrink-0 rounded-md overflow-hidden bg-[#0e0e18] shadow-md">
                      <Image 
                        src={movie.poster_path ? getTMDBImageUrl(movie.poster_path, "w92") : "/placeholder.svg"} 
                        alt={movie.title}
                        fill
                        className="object-cover"
                      />
                    </div>
                    
                    <div className="flex-1">
                      <h3 className="font-semibold text-base text-white tracking-wide">
                        {movie.title}
                      </h3>
                      <p className="text-sm text-[#8a8aaa] mt-0.5">{movie.original_title}</p>
                      
                      <div className="flex items-center gap-2 mt-3">
                        <Badge variant="outline" className="text-xs bg-[#2a2a40]/50 border-[#3d3d5c] px-2 py-0.5 rounded-md">
                          <Film className="h-3 w-3 mr-1" /> 电影
                        </Badge>
                        {movie.release_date && (
                          <Badge variant="outline" className="text-xs bg-[#2a2a40]/50 border-[#3d3d5c] px-2 py-0.5 rounded-md">
                            <Calendar className="h-3 w-3 mr-1" /> {movie.release_date.substring(0, 4)}
                          </Badge>
                        )}
                        {movie.vote_average > 0 && (
                          <Badge variant="outline" className="text-xs bg-[#2a2a40]/50 border-[#3d3d5c] px-2 py-0.5 rounded-md">
                            <Star className="h-3 w-3 mr-1 text-amber-400" /> {movie.vote_average.toFixed(1)}
                          </Badge>
                        )}
                        <Link
                          href={`https://www.themoviedb.org/movie/${movie.id}`}
                          target="_blank"
                          className="text-xs text-purple-400 hover:text-purple-300 ml-auto transition-colors"
                          onClick={(e) => e.stopPropagation()}
                        >
                          查看详情
                        </Link>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </TabsContent>
        </Tabs>
        
        <DialogFooter className="mt-4 pt-4 border-t border-[#2a2a40] px-6 pb-6 gap-3">
          <div className="flex-1">
            <div className="text-sm text-[#8a8aaa]">
              {!anilistId && (
                <div className="flex items-center text-amber-400 mb-2">
                  <AlertTriangle className="h-4 w-4 mr-2" />
                  <span>警告: 未找到AniList ID，确认操作可能无法完成</span>
                </div>
              )}
              {(selectedTVId || selectedMovieId) && (
                <div className="flex items-center">
                  <div className="w-2 h-2 rounded-full bg-purple-500 mr-2"></div>
                  <span>已选择: <span className="text-purple-400 font-medium">{getSelectedName()}</span></span>
                </div>
              )}
            </div>
          </div>
          <Button 
            variant="outline" 
            onClick={() => setIsOpen(false)} 
            className="bg-transparent text-white border-[#3d3d5c] hover:bg-[#2a2a40] transition-colors"
          >
            取消
          </Button>
          <Button 
            onClick={handleSaveMapping}
            disabled={(!selectedMovieId && !(selectedTVId && selectedSeasonId)) || isSaving || !anilistId}
            className="bg-gradient-to-r from-[#8a2be2] to-[#4169e1] hover:opacity-90 text-white shadow-md transition-all duration-200"
          >
            {isSaving ? <Loader2 className="h-4 w-4 animate-spin mr-2" /> : null}
            确认选择
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
} 