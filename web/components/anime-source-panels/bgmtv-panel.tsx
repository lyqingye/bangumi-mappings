"use client"

import { motion } from "framer-motion"
import Image from "next/image"
import Link from "next/link"
import { useState } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ChevronDown, ExternalLink, Search } from "lucide-react"
import { getBGMTVImageUrl, type BGMTVAnimeDetail } from "@/lib/api/bgmtv"
import type { Mapping, Platform } from "@/lib/types"
import { getStatusLabel, formatDate } from "@/lib/utils"
import { BGMTVSearchDialog } from "./bgmtv-search-dialog"

// 定义组件接收到的mapping结构
interface MappingObject {
  anilist_id: number;
  mappings: Mapping[];
  platformIds: Record<Platform, string | null>;
}

interface BGMTVPanelProps {
  data: BGMTVAnimeDetail | null | undefined;
  delay?: number;
  mapping?: MappingObject | null;
  anilistId?: number | null;
  onStatusUpdated?: () => void;
}

export function BGMTVPanel({ data, delay = 0.2, mapping, anilistId, onStatusUpdated }: BGMTVPanelProps) {
  const [summaryExpanded, setSummaryExpanded] = useState(false)
  const [searchDialogOpen, setSearchDialogOpen] = useState(false)

  // 找到BGM平台的映射
  const bgmMapping = mapping?.mappings.find(m => m.platform === "BgmTv") || null;
  
  const id = data?.id
  const title = data?.name || "Unknown Title"
  const titleCn = data?.name_cn || ""
  const coverImage = data?.images?.large ? getBGMTVImageUrl(data.images.large) : ""
  const airDate = data ? formatDate(data.date) : "Unknown"
  const episodes = data?.eps_count || data?.eps
  const type = data?.type === 2 ? "Anime" : "Other"
  const score = data?.rating?.score ? data.rating.score.toFixed(1) : "N/A"
  const tags = data?.tags?.map((t) => t.name) || []

  return (
    <>
      <motion.div
        className="bg-[#111] border border-[#222] rounded-lg overflow-hidden h-[450px] flex flex-col"
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.5, delay }}
      >
        <div className="p-4 bg-green-900/20 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Badge className="bg-green-600 text-white">BgmTV</Badge>

            {/* 显示验证状态 */}
            {bgmMapping && bgmMapping.review_status && (
              <Badge className={`${getStatusLabel(bgmMapping.review_status).color} text-white ml-2`}>
                {getStatusLabel(bgmMapping.review_status).label}
              </Badge>
            )}
          </div>
          <div className="flex items-center gap-2">
            <Button 
              variant="outline" 
              size="sm" 
              className="bg-transparent border-green-600 text-green-400 hover:bg-green-900/30 hover:text-green-300"
              onClick={() => setSearchDialogOpen(true)}
            >
              <Search className="h-3.5 w-3.5 mr-1" />
              搜索匹配
            </Button>
            {id && (
              <Link
                href={`https://bgm.tv/subject/${id}`}
                target="_blank"
                className="text-green-400 hover:text-green-300 text-sm flex items-center gap-1"
              >
                查看原页面 <ExternalLink className="h-3 w-3" />
              </Link>
            )}
          </div>
        </div>

        <div className="p-4 flex-1 overflow-y-auto scrollbar-hide">
          {data ? (
            <div className="flex gap-4">
              <div className="w-24 h-36 bg-[#222] rounded-md relative flex-shrink-0">
                <Image src={coverImage || "/placeholder.svg"} alt={title} fill className="object-cover rounded-md" />
              </div>

              <div className="flex-1">
                <h3 className="text-xl font-bold mb-1">
                  {titleCn || title}
                </h3>
                <p className="text-[#777] text-sm mb-4">
                  {title}
                </p>

                <div
                  className="flex items-center justify-between mb-2 cursor-pointer"
                  onClick={() => setSummaryExpanded((prev) => !prev)}
                >
                  <span className="text-sm font-medium">Summary</span>
                  <ChevronDown
                    className={`h-4 w-4 text-[#777] transition-transform ${summaryExpanded ? "rotate-180" : ""}`}
                  />
                </div>
                {summaryExpanded && (
                  <motion.div
                    className="mb-4 text-sm text-[#777]"
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: "auto" }}
                    exit={{ opacity: 0, height: 0 }}
                    transition={{ duration: 0.3 }}
                  >
                    {data.summary || "No summary available."}
                  </motion.div>
                )}

                <div className="grid grid-cols-2 gap-x-4 gap-y-2 mb-4">
                  <div>
                    <div className="text-xs text-[#777]">Air Date</div>
                    <div className="text-sm">{airDate}</div>
                  </div>
                  <div>
                    <div className="text-xs text-[#777]">Episodes</div>
                    <div className="text-sm">{episodes}</div>
                  </div>
                  <div>
                    <div className="text-xs text-[#777]">Type</div>
                    <div className="text-sm">{type}</div>
                  </div>
                  <div>
                    <div className="text-xs text-[#777]">Rating</div>
                    <div className="text-sm">{score} / 10</div>
                  </div>
                </div>

                <div>
                  <div className="text-xs text-[#777] mb-2">Tags</div>
                  <div className="flex flex-wrap gap-1">
                    {tags.slice(0, 10).map((tag, index) => (
                      <Badge
                        key={`bgmtv-tag-${index}`}
                        variant="secondary"
                        className="bg-[#222] text-white border-none text-xs"
                      >
                        {tag}
                      </Badge>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <div className="h-full flex flex-col items-center justify-center text-[#777]">
              <p>暂无 BgmTV 数据</p>
            </div>
          )}
        </div>
      </motion.div>

      {/* 搜索对话框 */}
      <BGMTVSearchDialog 
        isOpen={searchDialogOpen}
        setIsOpen={setSearchDialogOpen}
        anilistId={anilistId ?? null}
        onMappingSuccess={onStatusUpdated}
      />
    </>
  )
}

