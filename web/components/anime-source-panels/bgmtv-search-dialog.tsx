"use client"

import { useState, useEffect } from "react"
import { Search, Loader2, Star, Calendar, PlaySquare } from "lucide-react"
import Image from "next/image"
import Link from "next/link"
import { useToast } from "@/hooks/use-toast"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { searchBGMTV, getBGMTVImageUrl, type BGMTVAnimeDetail } from "@/lib/api/bgmtv"
import { manualMapping } from "@/lib/api/animes"
import { Platform } from "@/lib/types"

interface BGMTVSearchDialogProps {
  isOpen: boolean
  setIsOpen: (open: boolean) => void
  anilistId: number | null
  onMappingSuccess?: () => void
}

export function BGMTVSearchDialog({ isOpen, setIsOpen, anilistId, onMappingSuccess }: BGMTVSearchDialogProps) {
  const [searchTerm, setSearchTerm] = useState("")
  const [searchResults, setSearchResults] = useState<BGMTVAnimeDetail[]>([])
  const [isSearching, setIsSearching] = useState(false)
  const [selectedAnime, setSelectedAnime] = useState<BGMTVAnimeDetail | null>(null)
  const [isSaving, setIsSaving] = useState(false)
  const { toast } = useToast()

  // 清空选中的动画，当对话框关闭时
  useEffect(() => {
    if (!isOpen) {
      setSelectedAnime(null)
      setSearchResults([])
      setSearchTerm("")
    }
  }, [isOpen])

  // 处理搜索逻辑
  const handleSearch = async () => {
    if (!searchTerm.trim()) return
    
    setIsSearching(true)
    try {
      const response = await searchBGMTV({
        keyword: searchTerm.trim(),
        sort: 'rank',
        filter: {
          type: [2] // 只搜索动画类型
        }
      }, 10, 0)
      
      setSearchResults(response.data)
    } catch (error) {
      toast({
        title: "搜索失败",
        description: "无法获取BgmTV搜索结果，请稍后重试",
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

  const handleSaveMapping = async () => {
    if (!anilistId) {
      console.error("缺少anilistId，无法进行匹配")
      toast({
        title: "匹配失败",
        description: "缺少必要的番剧ID信息",
        variant: "destructive"
      })
      return
    }
    
    if (!selectedAnime) {
      console.error("未选择要匹配的动画")
      toast({
        title: "请选择一个匹配项",
        description: "请从搜索结果中选择一个要匹配的条目",
        variant: "destructive"
      })
      return
    }
    
    setIsSaving(true)
    try {
      await manualMapping(
        anilistId,
        Platform.BgmTv,
        selectedAnime.id.toString(),
        null // 暂不支持季度设置
      )
      
      toast({
        title: "匹配成功",
        description: "已成功将番剧与BgmTV条目关联",
        variant: "default"
      })
      
      onMappingSuccess?.()
      setIsOpen(false)
    } catch (error) {
      toast({
        title: "保存失败",
        description: "无法保存关联信息，请稍后重试",
        variant: "destructive"
      })
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogContent className="sm:max-w-[700px] bg-black border border-[#2a2a40] text-white max-h-[90vh] overflow-hidden flex flex-col shadow-xl rounded-xl">
        <DialogHeader className="border-b border-[#2a2a40] pb-4 px-6 pt-6">
          <DialogTitle className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-purple-400 to-blue-500">BgmTV 搜索 {anilistId ? `(ID: ${anilistId})` : ""}</DialogTitle>
        </DialogHeader>
        
        <div className="flex items-center gap-3 my-6 px-6">
          <div className="relative flex-1">
            <Input 
              placeholder="输入番剧名称搜索..." 
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
        
        <div className="flex-1 overflow-y-auto pr-2 min-h-[300px] px-6">
          {isSearching ? (
            <div className="h-full flex items-center justify-center">
              <div className="flex flex-col items-center">
                <Loader2 className="h-10 w-10 animate-spin text-purple-500 mb-3" />
                <p className="text-[#8a8aaa] text-sm">搜索中...</p>
              </div>
            </div>
          ) : searchResults.length === 0 ? (
            <div className="h-full flex items-center justify-center">
              <div className="flex flex-col items-center text-center px-6">
                <Search className="h-12 w-12 text-[#2a2a40] mb-4" />
                <p className="text-[#8a8aaa] text-sm max-w-xs">
                  {searchTerm.trim() ? "未找到相关番剧，请尝试其他关键词" : "请输入番剧名称开始搜索"}
                </p>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              {searchResults.map((anime) => (
                <div 
                  key={anime.id}
                  className={`p-4 rounded-xl cursor-pointer transition-all duration-200 flex gap-4 ${
                    selectedAnime?.id === anime.id 
                      ? "bg-[#121218] border-l-4 border-l-purple-500 border-t border-r border-b border-[#2a2a40]" 
                      : "border border-[#2a2a40] bg-[#121218] hover:border-purple-500/50 hover:bg-[#0c0c14]"
                  }`}
                  onClick={() => setSelectedAnime(anime)}
                >
                  <div className="w-16 h-24 relative flex-shrink-0 rounded-md overflow-hidden bg-[#0e0e18] shadow-md">
                    <Image 
                      src={anime.images?.large ? getBGMTVImageUrl(anime.images.large) : "/placeholder.svg"} 
                      alt={anime.name}
                      fill
                      className="object-cover"
                    />
                  </div>
                  
                  <div className="flex-1">
                    <h3 className="font-semibold text-base text-white tracking-wide">
                      {anime.name_cn || anime.name}
                    </h3>
                    <p className="text-sm text-[#8a8aaa] mt-0.5">{anime.name}</p>
                    
                    <div className="flex flex-wrap items-center gap-2 mt-3">
                      <Badge variant="outline" className="text-xs bg-[#2a2a40]/50 border-[#3d3d5c] px-2 py-0.5 rounded-md">
                        <PlaySquare className="h-3 w-3 mr-1" /> {anime.eps_count || anime.eps || "未知"} 集
                      </Badge>
                      <Badge variant="outline" className="text-xs bg-[#2a2a40]/50 border-[#3d3d5c] px-2 py-0.5 rounded-md">
                        <Calendar className="h-3 w-3 mr-1" /> {anime.date || "日期未知"}
                      </Badge>
                      {anime.rating?.score && (
                        <Badge variant="outline" className="text-xs bg-[#2a2a40]/50 border-[#3d3d5c] px-2 py-0.5 rounded-md">
                          <Star className="h-3 w-3 mr-1 text-amber-400" /> {anime.rating.score.toFixed(1)}
                        </Badge>
                      )}
                      <Link
                        href={`https://bgm.tv/subject/${anime.id}`}
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
        </div>
        
        <DialogFooter className="mt-4 pt-4 border-t border-[#2a2a40] px-6 pb-6 gap-3">
          <div className="flex-1 text-sm text-[#8a8aaa]">
            {selectedAnime && (
              <div className="flex items-center">
                <div className="w-2 h-2 rounded-full bg-purple-500 mr-2"></div>
                <span>已选择: <span className="text-purple-400 font-medium">{selectedAnime.name_cn || selectedAnime.name}</span></span>
              </div>
            )}
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
            disabled={!selectedAnime || isSaving || !anilistId}
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