"use client"

import { useState, useEffect } from "react"
import { motion } from "framer-motion"
import { Search, Loader2 } from "lucide-react"
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
      <DialogContent className="sm:max-w-[700px] bg-[#111] border border-[#222] text-white max-h-[90vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="text-xl">BgmTV 搜索 {anilistId ? `(ID: ${anilistId})` : ""}</DialogTitle>
        </DialogHeader>
        
        <div className="flex items-center gap-2 my-4">
          <Input 
            placeholder="输入番剧名称搜索..." 
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="bg-[#222] border-[#333] text-white"
            onKeyDown={handleKeyDown}
          />
          <Button 
            onClick={handleSearch}
            disabled={isSearching || !searchTerm.trim()}
            className="bg-green-600 hover:bg-green-700"
          >
            {isSearching ? <Loader2 className="h-4 w-4 animate-spin" /> : <Search className="h-4 w-4" />}
            搜索
          </Button>
        </div>
        
        <div className="flex-1 overflow-y-auto pr-2 min-h-[300px]">
          {isSearching ? (
            <div className="h-full flex items-center justify-center">
              <Loader2 className="h-8 w-8 animate-spin text-green-500" />
            </div>
          ) : searchResults.length === 0 ? (
            <div className="h-full flex items-center justify-center text-[#777]">
              {searchTerm.trim() ? "未找到相关番剧" : "请输入番剧名称开始搜索"}
            </div>
          ) : (
            <div className="space-y-4">
              {searchResults.map((anime) => (
                <div 
                  key={anime.id}
                  className={`p-3 rounded-md cursor-pointer transition-colors flex gap-3 ${
                    selectedAnime?.id === anime.id 
                      ? "border-2 border-green-500 bg-green-950/20" 
                      : "border border-[#333] hover:bg-[#1a1a1a]"
                  }`}
                  onClick={() => setSelectedAnime(anime)}
                >
                  <div className="w-16 h-24 relative flex-shrink-0 rounded overflow-hidden bg-[#222]">
                    <Image 
                      src={anime.images?.large ? getBGMTVImageUrl(anime.images.large) : "/placeholder.svg"} 
                      alt={anime.name}
                      fill
                      className="object-cover"
                    />
                  </div>
                  
                  <div className="flex-1">
                    <h3 className="font-medium">
                      {anime.name_cn || anime.name}
                    </h3>
                    <p className="text-sm text-[#777]">{anime.name}</p>
                    
                    <div className="flex items-center gap-2 mt-2">
                      <Badge variant="outline" className="text-xs bg-[#222]">
                        {anime.eps_count || anime.eps || "未知"} 集
                      </Badge>
                      <Badge variant="outline" className="text-xs bg-[#222]">
                        {anime.date || "日期未知"}
                      </Badge>
                      {anime.rating?.score && (
                        <Badge variant="outline" className="text-xs bg-[#222]">
                          评分: {anime.rating.score.toFixed(1)}
                        </Badge>
                      )}
                      <Link
                        href={`https://bgm.tv/subject/${anime.id}`}
                        target="_blank"
                        className="text-xs text-green-400 hover:text-green-300 ml-auto"
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
        
        <DialogFooter className="mt-4 pt-4 border-t border-[#333]">
          <Button variant="outline" onClick={() => setIsOpen(false)} className="bg-transparent text-white border-[#333] hover:bg-[#333]">
            取消
          </Button>
          <Button 
            onClick={handleSaveMapping}
            disabled={!selectedAnime || isSaving || !anilistId}
            className="bg-green-600 hover:bg-green-700"
          >
            {isSaving ? <Loader2 className="h-4 w-4 animate-spin mr-2" /> : null}
            确认选择
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
} 