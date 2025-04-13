"use client"

import { motion } from "framer-motion"
import { useState } from "react"
import Image from "next/image"
import Link from "next/link"
import { Badge } from "@/components/ui/badge"
import { ChevronDown, ExternalLink } from "lucide-react"
import { getTMDBImageUrl, type TMDBAnimeDetail } from "@/lib/api/tmdb"
import type { Mapping, Platform } from "@/lib/types"
import { getStatusLabel, formatDate } from "@/lib/utils"

// 定义组件接收到的mapping结构
interface MappingObject {
  anilist_id: number;
  mappings: Mapping[];
  platformIds: Record<Platform, string | null>;
}

interface TMDBPanelProps {
  detail: TMDBAnimeDetail
  delay?: number
  mapping?: MappingObject | null
  onStatusUpdated?: () => void
}

export function TMDBPanel({ detail, delay = 0.2, mapping }: TMDBPanelProps) {
  const [summaryExpanded, setSummaryExpanded] = useState(false)

  // 找到TMDB平台的映射
  const tmdbMapping = mapping?.mappings.find(m => m.platform === "Tmdb") || null;

  if (!detail) return null

  const id = detail.id
  const title = detail.name || "Unknown Title"
  const originalTitle = detail.original_name || ""
  const coverImage = detail.poster_path ? getTMDBImageUrl(detail.poster_path, "w500") : ""
  const firstAirDate = formatDate(detail.first_air_date) || "Unknown"
  const episodes = detail.number_of_episodes || 0
  const type = detail.type || "TV"
  const score = detail.vote_average ? detail.vote_average.toFixed(1) : "N/A"
  const genres = detail.genres?.map((g) => g.name) || []

  return (
    <motion.div
      className="bg-[#111] border border-[#222] rounded-lg overflow-hidden h-[450px] flex flex-col"
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.5, delay }}
    >
      <div className="p-4 bg-blue-900/20 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Badge className="bg-blue-600 text-white">TMDB</Badge>

          {/* 显示验证状态 */}
          {tmdbMapping && tmdbMapping.review_status && (
            <Badge className={`${getStatusLabel(tmdbMapping.review_status).color} text-white ml-2`}>
              {getStatusLabel(tmdbMapping.review_status).label}
            </Badge>
          )}
        </div>
        <Link
          href={`https://www.themoviedb.org/tv/${id}`}
          target="_blank"
          className="text-blue-400 hover:text-blue-300 text-sm flex items-center gap-1"
        >
          View on TMDB <ExternalLink className="h-3 w-3" />
        </Link>
      </div>

      <div className="p-4 flex-1 overflow-y-auto scrollbar-hide">
        <div className="flex gap-4">
          <div className="w-24 h-36 bg-[#222] rounded-md relative flex-shrink-0">
            <Image src={coverImage || "/placeholder.svg"} alt={title} fill className="object-cover rounded-md" />
          </div>

          <div className="flex-1">
            <h3 className="text-xl font-bold mb-1">
              {title}
            </h3>
            <p className="text-[#777] text-sm mb-4">
              {originalTitle}
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
                {detail.overview || "No summary available."}
              </motion.div>
            )}

            <div className="grid grid-cols-2 gap-x-4 gap-y-2 mb-4">
              <div>
                <div className="text-xs text-[#777]">Air Date</div>
                <div className="text-sm">{firstAirDate}</div>
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
              <div className="text-xs text-[#777] mb-2">Genres</div>
              <div className="flex flex-wrap gap-1">
                {genres.slice(0, 10).map((genre, index) => (
                  <Badge
                    key={`tmdb-genre-${index}`}
                    variant="secondary"
                    className="bg-[#222] text-white border-none text-xs"
                  >
                    {genre}
                  </Badge>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  )
}

